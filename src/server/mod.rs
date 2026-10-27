//! Process model.
//!
//! Clients start the binary over stdio as before, but by default it runs as a
//! thin **proxy**: it answers `initialize` / `tools/list` itself and forwards
//! tool calls over a Unix socket to a single shared **daemon** that owns the
//! only Telegram connection. The daemon is started on demand by the first
//! proxy that needs it and exits after a period with no clients; proxies drop
//! their daemon connection when idle, so idle client sessions don't keep it up.

pub mod daemon;
pub mod paths;
pub mod proxy;

use crate::config::AppConfig;
use crate::telegram::grammers_impl::GrammersService;
use crate::telegram::path_guard::PathGuard;
use crate::telegram::TelegramService;
use anyhow::{bail, Result};
use grammers_client::Client;
use grammers_mtsender::SenderPool;
use grammers_session::storages::SqliteSession;
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Resolves allowed roots for file tools: CLI args first, then
/// `TELEGRAM_ALLOWED_ROOTS` (comma separated).
pub fn resolve_roots(cli: &[PathBuf]) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = cli.iter().filter_map(|r| r.canonicalize().ok()).collect();
    if roots.is_empty() {
        if let Ok(env_roots) = std::env::var("TELEGRAM_ALLOWED_ROOTS") {
            roots = env_roots
                .split(',')
                .filter_map(|r| PathBuf::from(r.trim()).canonicalize().ok())
                .collect();
        }
    }
    roots
}

/// Opens the session, connects and checks authorization.
pub async fn start_telegram(roots: Vec<PathBuf>) -> Result<Arc<dyn TelegramService>> {
    let config = AppConfig::load()?;
    if roots.is_empty() {
        info!("No allowed roots configured. File tools will be disabled.");
    } else {
        info!("Allowed roots: {:?}", roots);
    }

    let session = Arc::new(SqliteSession::open(&config.session_file).await?);
    let SenderPool { runner, updates: _, handle } = SenderPool::new(Arc::clone(&session), config.api_id);
    tokio::spawn(runner.run());
    let client = Client::new(handle);

    if !client.is_authorized().await? {
        bail!("Telegram client is not authorized! Please run the login bin first.");
    }
    info!("Telegram client initialized and authorized.");

    Ok(Arc::new(GrammersService::new(client, session, PathGuard::new(roots))))
}
