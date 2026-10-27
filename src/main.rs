use anyhow::Result;
use clap::Parser;
use std::io::{self, IsTerminal};
use std::path::PathBuf;
use std::sync::Arc;
use telegram_mcp_rs::{
    mcp::{dispatcher::Dispatcher, protocol::JsonRpcRequest},
    server::{self, paths},
};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Run the shared Telegram daemon (started automatically by the proxy).
    #[arg(long)]
    daemon: bool,

    /// Build id the daemon serves (set by the proxy).
    #[arg(long, requires = "daemon")]
    build: Option<String>,

    /// Legacy mode: one process per client with its own Telegram connection.
    #[arg(long, conflicts_with = "daemon")]
    direct: bool,

    /// Optional positional arguments define server-side allowed roots for file-path tools.
    #[arg(value_name = "ALLOWED_ROOTS")]
    allowed_roots: Vec<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .with_ansi(io::stderr().is_terminal())
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Everything (.env, telegram.session, run/) lives in the install root,
    // whatever directory the client started us from.
    let exe = server::proxy::exe_path()?;
    let base = paths::base_dir(&exe);
    std::env::set_current_dir(&base)?;
    dotenvy::from_path(base.join(".env")).ok();
    let run = paths::RunPaths::new(&base)?;
    let roots = server::resolve_roots(&args.allowed_roots);

    let result = if args.daemon {
        let build = match args.build {
            Some(b) => b,
            None => paths::build_id(&exe)?,
        };
        server::daemon::run(build, roots, run).await
    } else if args.direct {
        run_direct(roots).await
    } else {
        server::proxy::run(exe, args.allowed_roots, run).await
    };
    if let Err(e) = &result {
        error!("{e:#}");
    }
    result
}

/// Original single-process stdio server.
async fn run_direct(roots: Vec<PathBuf>) -> Result<()> {
    let telegram = server::start_telegram(roots).await?;
    let dispatcher = Arc::new(Dispatcher::new(telegram));

    info!("Starting MCP JSON-RPC stdio loop...");
    let mut stdout = tokio::io::stdout();
    let mut lines = BufReader::new(tokio::io::stdin()).lines();
    while let Some(line) = lines.next_line().await? {
        if line.trim().is_empty() {
            continue;
        }
        match serde_json::from_str::<JsonRpcRequest>(&line) {
            Ok(req) => {
                if let Some(resp) = dispatcher.dispatch(req).await {
                    let out = serde_json::to_string(&resp)?;
                    stdout.write_all(out.as_bytes()).await?;
                    stdout.write_all(b"\n").await?;
                    stdout.flush().await?;
                }
            }
            Err(e) => error!("Failed to parse JSON-RPC request: {}", e),
        }
    }
    Ok(())
}
