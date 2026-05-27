use anyhow::Result;
use clap::Parser;
use grammers_client::Client;
use grammers_mtsender::SenderPool;
use grammers_session::storages::SqliteSession;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::Arc;
use telegram_mcp_rs::{
    config::AppConfig,
    mcp::{dispatcher::Dispatcher, protocol::JsonRpcRequest, sse::SseServer},
    telegram::grammers_impl::GrammersService,
    telegram::path_guard::PathGuard,
};
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Optional positional arguments define server-side allowed roots for file-path tools.
    #[arg(value_name = "ALLOWED_ROOTS")]
    allowed_roots: Vec<PathBuf>,

    /// Run as an HTTP/SSE server on the specified port.
    #[arg(short, long)]
    sse: Option<u16>,

    /// Host to bind the SSE server to (default: 127.0.0.1).
    #[arg(short = 'H', long, default_value = "127.0.0.1")]
    host: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 0. Load .env
    dotenvy::dotenv().ok();

    // 1. Parse CLI arguments
    let args = Args::parse();

    // 2. Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .with_writer(io::stderr)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // 3. Load Configuration
    let config = match AppConfig::load() {
        Ok(c) => c,
        Err(e) => {
            error!("Configuration error: {:?}", e);
            std::process::exit(1);
        }
    };
    info!("Loaded configuration successfully.");

    // 4. Initialize PathGuard
    let mut resolved_roots = Vec::new();
    
    // 4a. Try loading from CLI arguments
    for root in args.allowed_roots {
        if root.exists() {
            resolved_roots.push(root.canonicalize()?);
        }
    }

    // 4b. Try loading from Environment Variable (fallback)
    if resolved_roots.is_empty() {
        if let Ok(env_roots) = std::env::var("TELEGRAM_ALLOWED_ROOTS") {
            for root_str in env_roots.split(',') {
                let root = std::path::PathBuf::from(root_str.trim());
                if root.exists() {
                    resolved_roots.push(root.canonicalize()?);
                }
            }
        }
    }

    if resolved_roots.is_empty() {
        info!("Warning: No allowed roots configured. File tools will be disabled.");
    } else {
        info!("Allowed roots: {:?}", resolved_roots);
    }
    
    let path_guard = PathGuard::new(resolved_roots);

    // 5. Initialize Telegram Session
    let session = Arc::new(SqliteSession::open(&config.session_file).await?);
    let SenderPool {
        runner,
        updates: _,
        handle,
    } = SenderPool::new(Arc::clone(&session), config.api_id);
    let _pool_task = tokio::spawn(runner.run());

    let client = Client::new(handle.clone());

    if !client.is_authorized().await? {
        error!("Telegram client is not authorized! Please run the login bin first.");
        std::process::exit(1);
    }

    info!("Telegram client initialized and authorized.");

    // 6. Wire up services
    let telegram_service = Arc::new(GrammersService::new(client, path_guard));
    let dispatcher = Arc::new(Dispatcher::new(telegram_service));

    if let Some(port) = args.sse {
        // 7a. MCP SSE Server mode
        let addr: std::net::SocketAddr = format!("{}:{}", args.host, port).parse()?;
        let server = Arc::new(SseServer::new(dispatcher));
        server.run(addr).await?;
    } else {
        // 7b. MCP stdio loop
        info!("Starting MCP JSON-RPC stdio loop...");
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = line?;
            if line.trim().is_empty() {
                continue;
            }

            match serde_json::from_str::<JsonRpcRequest>(&line) {
                Ok(req) => {
                    if let Some(resp) = dispatcher.dispatch(req).await {
                        let out = serde_json::to_string(&resp)?;
                        println!("{}", out);
                        let _ = io::stdout().flush();
                    }
                }
                Err(e) => {
                    error!("Failed to parse JSON-RPC request: {}", e);
                }
            }
        }
    }

    Ok(())
}
