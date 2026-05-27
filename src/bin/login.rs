use anyhow::{Context, Result};
use grammers_client::Client;
use grammers_mtsender::SenderPool;
use grammers_session::storages::SqliteSession;
use std::io::{self, Write};
use std::sync::Arc;
use telegram_mcp_rs::config::AppConfig;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Telegram Rust MCP Login ===");
    
    // Load config from .env
    dotenvy::dotenv().ok();
    let config = AppConfig::load().context("Failed to load config. Make sure TELEGRAM_API_ID and TELEGRAM_API_HASH are in .env")?;
    
    println!("API ID: {}", config.api_id);
    println!("Session File: {}", config.session_file);

    let session = Arc::new(SqliteSession::open(&config.session_file).await?);
    let SenderPool {
        runner,
        updates: _,
        handle,
    } = SenderPool::new(Arc::clone(&session), config.api_id);
    
    let client = Client::new(handle.clone());
    let _pool_task = tokio::spawn(runner.run());

    if !client.is_authorized().await? {
        println!("\nYou are not authorized.");
        let phone = prompt("Enter your phone number (e.g. +1234567890): ")?;
        
        println!("Requesting login code...");
        let token = client
            .request_login_code(&phone, &config.api_hash)
            .await
            .context("Failed to request login code")?;
            
        let code = prompt("Enter the code you received on Telegram: ")?;
        
        match client.sign_in(&token, &code).await {
            Ok(user) => {
                println!("Successfully signed in as {} (ID: {})!", user.first_name().unwrap_or(""), user.id());
            }
            Err(grammers_client::SignInError::PasswordRequired(password_token)) => {
                let password = prompt("Two-factor authentication enabled. Enter your password: ")?;
                client
                    .check_password(password_token, &password)
                    .await
                    .context("Failed to check password")?;
                println!("Successfully signed in with 2FA!");
            }
            Err(e) => {
                anyhow::bail!("Failed to sign in: {}", e);
            }
        }
    } else {
        let me = client.get_me().await?;
        println!("You are already authorized as {} (ID: {}).", me.first_name().unwrap_or(""), me.id());
    }

    // Ensure session is saved
    println!("Session saved to {}", config.session_file);
    
    // Graceful shutdown
    drop(client); // Drop the client to release the session file lock if any
    
    Ok(())
}

fn prompt(message: &str) -> Result<String> {
    print!("{}", message);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
