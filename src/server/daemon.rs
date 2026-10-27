//! The single process that talks to Telegram.
//!
//! Serves newline-delimited JSON-RPC (the same framing as MCP stdio) on a Unix
//! socket, handles requests from all connections concurrently, and exits once
//! it has had no connections for `IDLE_EXIT`.

use super::paths::RunPaths;
use crate::mcp::dispatcher::Dispatcher;
use crate::mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use anyhow::{Context, Result};
use serde_json::Value;
use std::fs::{File, TryLockError};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;
use tracing::{error, info, warn};

/// How long the daemon stays up with zero connected proxies.
fn idle_exit() -> Duration {
    let secs = std::env::var("TELEGRAM_DAEMON_IDLE_SECS").ok().and_then(|s| s.parse().ok()).unwrap_or(600);
    Duration::from_secs(secs)
}

struct Activity {
    connections: AtomicUsize,
    last_change: Mutex<Instant>,
}

impl Activity {
    fn touch(&self) {
        *self.last_change.lock().unwrap() = Instant::now();
    }
}

/// Decrements the connection count when a connection handler finishes.
struct ConnGuard(Arc<Activity>);

impl Drop for ConnGuard {
    fn drop(&mut self) {
        self.0.connections.fetch_sub(1, Ordering::SeqCst);
        self.0.touch();
    }
}

pub async fn run(build: String, roots: Vec<PathBuf>, run: RunPaths) -> Result<()> {
    // One daemon per build: hold this lock until exit.
    let lock_path = run.daemon_lock(&build);
    let lock = File::create(&lock_path).with_context(|| format!("open {}", lock_path.display()))?;
    match lock.try_lock() {
        Ok(()) => {}
        Err(TryLockError::WouldBlock) => {
            info!("Another daemon for build {build} is already running; exiting.");
            return Ok(());
        }
        Err(TryLockError::Error(e)) => return Err(e).context("lock daemon file"),
    }

    info!("Daemon {build} (pid {}) starting", std::process::id());
    remove_stale_locks(&run, &lock_path);
    let telegram = super::start_telegram(roots).await?;
    let dispatcher = Arc::new(Dispatcher::new(telegram));

    let socket = run.socket(&build);
    let _ = std::fs::remove_file(&socket);
    let listener = UnixListener::bind(&socket).with_context(|| format!("bind {}", socket.display()))?;
    std::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600))?;
    info!("Listening on {}", socket.display());

    let activity = Arc::new(Activity { connections: AtomicUsize::new(0), last_change: Mutex::new(Instant::now()) });
    let idle = idle_exit();
    let mut tick = tokio::time::interval(Duration::from_secs(5));
    let mut sigterm = signal(SignalKind::terminate())?;
    let mut sigint = signal(SignalKind::interrupt())?;

    loop {
        tokio::select! {
            accepted = listener.accept() => match accepted {
                Ok((stream, _)) => {
                    activity.connections.fetch_add(1, Ordering::SeqCst);
                    activity.touch();
                    let guard = ConnGuard(Arc::clone(&activity));
                    let dispatcher = Arc::clone(&dispatcher);
                    tokio::spawn(async move {
                        serve_connection(stream, dispatcher).await;
                        drop(guard);
                    });
                }
                Err(e) => warn!("accept failed: {e}"),
            },
            _ = tick.tick() => {
                let idle_for = activity.last_change.lock().unwrap().elapsed();
                if activity.connections.load(Ordering::SeqCst) == 0 && idle_for >= idle {
                    info!("No clients for {}s; exiting.", idle_for.as_secs());
                    break;
                }
            }
            _ = sigterm.recv() => { info!("SIGTERM; exiting."); break; }
            _ = sigint.recv() => { info!("SIGINT; exiting."); break; }
        }
    }

    // Unlink first so new proxies start a fresh daemon instead of connecting here.
    let _ = std::fs::remove_file(&socket);
    drop(listener);
    drop(lock);
    Ok(())
}

/// Deletes lock files of other builds whose daemons are gone (lock is free).
fn remove_stale_locks(run: &RunPaths, own: &std::path::Path) {
    let Ok(entries) = std::fs::read_dir(&run.dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or_default();
        if path == own || !(name.starts_with("daemon-") && name.ends_with(".lock")) {
            continue;
        }
        if let Ok(f) = File::open(&path) {
            if f.try_lock().is_ok() {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

async fn serve_connection(stream: UnixStream, dispatcher: Arc<Dispatcher>) {
    let (read_half, mut write_half) = stream.into_split();
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();

    let writer = tokio::spawn(async move {
        while let Some(line) = rx.recv().await {
            if write_half.write_all(line.as_bytes()).await.is_err() || write_half.write_all(b"\n").await.is_err() {
                break;
            }
        }
    });

    let mut lines = BufReader::new(read_half).lines();
    loop {
        let line = match lines.next_line().await {
            Ok(Some(line)) => line,
            Ok(None) => break,
            Err(e) => {
                warn!("read failed: {e}");
                break;
            }
        };
        if line.trim().is_empty() {
            continue;
        }
        let tx = tx.clone();
        let dispatcher = Arc::clone(&dispatcher);
        // Requests run concurrently; responses are matched by id on the other side.
        tokio::spawn(async move {
            let resp = match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => dispatcher.dispatch(req).await,
                Err(e) => {
                    error!("Bad JSON-RPC from proxy: {e}");
                    Some(JsonRpcResponse {
                        jsonrpc: "2.0".into(),
                        id: Value::Null,
                        result: None,
                        error: Some(JsonRpcError { code: -32700, message: format!("Parse error: {e}"), data: None }),
                    })
                }
            };
            if let Some(resp) = resp {
                if let Ok(out) = serde_json::to_string(&resp) {
                    let _ = tx.send(out);
                }
            }
        });
    }
    // Let in-flight requests finish and flush before closing.
    drop(tx);
    let _ = writer.await;
}
