//! The per-client stdio process. Holds no Telegram connection.
//!
//! `initialize`, `tools/list` and `ping` are answered locally from the static
//! catalog; `tools/call` is forwarded to the daemon, which is started on demand.
//! The daemon connection is opened lazily and dropped after `IDLE_DISCONNECT`
//! without traffic, so an idle client session does not keep the daemon alive.

use super::paths::{build_id, RunPaths};
use crate::mcp::dispatcher::{Catalog, ToolsMode};
use crate::mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};
use std::collections::HashSet;
use std::fs::OpenOptions;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::unix::OwnedWriteHalf;
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tracing::{info, warn};

const IDLE_DISCONNECT: Duration = Duration::from_secs(60);
const DAEMON_START_TIMEOUT: Duration = Duration::from_secs(60);

type Pending = Arc<Mutex<HashSet<String>>>;

struct Conn {
    writer: OwnedWriteHalf,
    alive: Arc<AtomicBool>,
    pending: Pending,
    last_used: Instant,
}

pub struct Proxy {
    exe: PathBuf,
    roots: Vec<PathBuf>,
    run: RunPaths,
    catalog: Catalog,
    out: mpsc::UnboundedSender<String>,
    conn: Option<Conn>,
}

/// Key for matching responses to requests (ids may be numbers or strings).
fn id_key(id: &Value) -> String {
    id.to_string()
}

fn error_line(id: Value, message: String) -> String {
    let resp = JsonRpcResponse {
        jsonrpc: "2.0".into(),
        id,
        result: None,
        error: Some(JsonRpcError { code: -32000, message, data: None }),
    };
    serde_json::to_string(&resp).unwrap_or_default()
}

pub async fn run(exe: PathBuf, roots: Vec<PathBuf>, run: RunPaths) -> Result<()> {
    let (out, mut out_rx) = mpsc::unbounded_channel::<String>();
    // Single stdout writer so concurrent responses never interleave.
    let stdout_task = tokio::spawn(async move {
        let mut stdout = tokio::io::stdout();
        while let Some(line) = out_rx.recv().await {
            if stdout.write_all(line.as_bytes()).await.is_err()
                || stdout.write_all(b"\n").await.is_err()
                || stdout.flush().await.is_err()
            {
                break;
            }
        }
    });

    let mut proxy = Proxy { exe, roots, run, catalog: Catalog::new(ToolsMode::from_env()), out, conn: None };
    let mut stdin = BufReader::new(tokio::io::stdin()).lines();
    let mut tick = tokio::time::interval(Duration::from_secs(10));

    loop {
        tokio::select! {
            line = stdin.next_line() => match line {
                Ok(Some(line)) => proxy.handle_line(line).await,
                // Client went away: we're done.
                Ok(None) | Err(_) => break,
            },
            _ = tick.tick() => proxy.drop_idle_conn(),
        }
    }

    // Give in-flight calls a moment to answer before exiting.
    let deadline = Instant::now() + Duration::from_secs(5);
    while proxy.has_pending() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
    drop(proxy);
    let _ = stdout_task.await;
    Ok(())
}

impl Proxy {
    async fn handle_line(&mut self, line: String) {
        if line.trim().is_empty() {
            return;
        }
        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                warn!("Failed to parse JSON-RPC request: {e}");
                return;
            }
        };
        let Some(id) = req.id.clone() else {
            // Notifications (initialized, cancelled, ...) need no answer.
            return;
        };
        let local = match req.method.as_str() {
            "initialize" => Some(self.catalog.initialize_result()),
            "tools/list" => Some(self.catalog.tools_list_result()),
            "ping" => Some(json!({})),
            _ => None,
        };
        if let Some(result) = local {
            let resp = JsonRpcResponse { jsonrpc: "2.0".into(), id, result: Some(result), error: None };
            let _ = self.out.send(serde_json::to_string(&resp).unwrap_or_default());
            return;
        }

        if let Err(e) = self.forward(&id, &line).await {
            let _ = self.out.send(error_line(id, format!("Telegram daemon unavailable: {e:#}")));
        }
    }

    /// Sends one request line to the daemon. A failed write means the request
    /// never reached it, so retrying once on a fresh connection is safe.
    async fn forward(&mut self, id: &Value, line: &str) -> Result<()> {
        for attempt in 0..2 {
            let conn = self.ensure_conn().await?;
            conn.pending.lock().unwrap().insert(id_key(id));
            conn.last_used = Instant::now();
            let mut buf = line.as_bytes().to_vec();
            buf.push(b'\n');
            match conn.writer.write_all(&buf).await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    conn.pending.lock().unwrap().remove(&id_key(id));
                    conn.alive.store(false, Ordering::SeqCst);
                    self.conn = None;
                    if attempt == 1 {
                        return Err(e.into());
                    }
                }
            }
        }
        unreachable!()
    }

    fn has_pending(&self) -> bool {
        self.conn.as_ref().is_some_and(|c| c.alive.load(Ordering::SeqCst) && !c.pending.lock().unwrap().is_empty())
    }

    fn drop_idle_conn(&mut self) {
        let drop_it = match &self.conn {
            Some(c) => !c.alive.load(Ordering::SeqCst) || (c.pending.lock().unwrap().is_empty() && c.last_used.elapsed() >= IDLE_DISCONNECT),
            None => false,
        };
        if drop_it {
            // Dropping the write half closes our side; the daemon sees EOF.
            self.conn = None;
        }
    }

    async fn ensure_conn(&mut self) -> Result<&mut Conn> {
        if self.conn.as_ref().is_some_and(|c| !c.alive.load(Ordering::SeqCst)) {
            self.conn = None;
        }
        if self.conn.is_none() {
            let stream = self.connect_or_spawn().await?;
            self.conn = Some(self.attach(stream));
        }
        Ok(self.conn.as_mut().unwrap())
    }

    /// Starts the reader that relays daemon responses to stdout.
    fn attach(&self, stream: UnixStream) -> Conn {
        let (read_half, writer) = stream.into_split();
        let alive = Arc::new(AtomicBool::new(true));
        let pending: Pending = Arc::new(Mutex::new(HashSet::new()));
        let out = self.out.clone();
        let (alive_r, pending_r) = (Arc::clone(&alive), Arc::clone(&pending));
        tokio::spawn(async move {
            let mut lines = BufReader::new(read_half).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if let Ok(v) = serde_json::from_str::<Value>(&line) {
                    if let Some(id) = v.get("id") {
                        pending_r.lock().unwrap().remove(&id_key(id));
                    }
                }
                let _ = out.send(line);
            }
            // Daemon went away: fail whatever was still waiting.
            alive_r.store(false, Ordering::SeqCst);
            let lost: Vec<String> = pending_r.lock().unwrap().drain().collect();
            for key in lost {
                let id = serde_json::from_str(&key).unwrap_or(Value::Null);
                let _ = out.send(error_line(id, "Connection to Telegram daemon lost; please retry.".into()));
            }
        });
        Conn { writer, alive, pending, last_used: Instant::now() }
    }

    async fn connect_or_spawn(&self) -> Result<UnixStream> {
        // Re-read the build every time: after a rebuild, reconnects go to the new code.
        let build = build_id(&self.exe)?;
        let socket = self.run.socket(&build);
        if let Ok(s) = UnixStream::connect(&socket).await {
            return Ok(s);
        }

        // Serialize spawning across proxies; whoever gets the lock second will
        // find the socket already up.
        let lock_path = self.run.spawn_lock();
        let lock = tokio::task::spawn_blocking(move || -> Result<std::fs::File> {
            let f = std::fs::File::create(&lock_path)?;
            f.lock()?;
            Ok(f)
        })
        .await??;

        if let Ok(s) = UnixStream::connect(&socket).await {
            return Ok(s);
        }
        self.spawn_daemon(&build)?;

        let deadline = Instant::now() + DAEMON_START_TIMEOUT;
        let result = loop {
            if let Ok(s) = UnixStream::connect(&socket).await {
                break Ok(s);
            }
            if Instant::now() >= deadline {
                break Err(anyhow!("daemon did not come up in {}s; see {}", DAEMON_START_TIMEOUT.as_secs(), self.run.log().display()));
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        };
        drop(lock);
        result
    }

    fn spawn_daemon(&self, build: &str) -> Result<()> {
        let log = OpenOptions::new().create(true).append(true).open(self.run.log())?;
        let mut cmd = std::process::Command::new(&self.exe);
        cmd.arg("--daemon").arg("--build").arg(build).args(&self.roots);
        cmd.stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::from(log));
        // Own session: not tied to this client's process group or terminal.
        unsafe {
            cmd.pre_exec(|| {
                libc::setsid();
                Ok(())
            });
        }
        let mut child = cmd.spawn().with_context(|| format!("spawn {}", self.exe.display()))?;
        info!("Started Telegram daemon pid {} (build {build})", child.id());
        // Reap it if it exits while we're still alive; otherwise init adopts it.
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}

/// Path the proxy uses to (re)spawn the daemon. Resolved once at startup:
/// after a rebuild `/proc/self/exe` points at a deleted inode, but this path
/// points at the new binary.
pub fn exe_path() -> Result<PathBuf> {
    let exe = std::env::current_exe()?;
    Ok(exe.canonicalize().unwrap_or(exe))
}
