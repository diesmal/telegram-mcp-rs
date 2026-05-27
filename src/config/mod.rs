use anyhow::{Context, Result};
use std::env;

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub api_id: i32,
    pub api_hash: String,
    pub session_file: String,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        let api_id_str = env::var("TELEGRAM_API_ID")
            .context("TELEGRAM_API_ID is required")?;
        let api_id = api_id_str
            .parse::<i32>()
            .context("TELEGRAM_API_ID must be a valid integer")?;

        let api_hash = env::var("TELEGRAM_API_HASH")
            .context("TELEGRAM_API_HASH is required")?;

        let session_file = env::var("TELEGRAM_SESSION_FILE")
            .unwrap_or_else(|_| "telegram.session".to_string());

        Ok(Self {
            api_id,
            api_hash,
            session_file,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Mutex to prevent environment variable race conditions in tests
    lazy_static::lazy_static! {
        static ref ENV_MUTEX: Mutex<()> = Mutex::new(());
    }

    #[test]
    fn test_load_config_success() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            env::set_var("TELEGRAM_API_ID", "12345");
            env::set_var("TELEGRAM_API_HASH", "abcdef");
            env::set_var("TELEGRAM_SESSION_FILE", "test.session");
        }

        let config = AppConfig::load().unwrap();
        assert_eq!(config.api_id, 12345);
        assert_eq!(config.api_hash, "abcdef");
        assert_eq!(config.session_file, "test.session");

        unsafe {
            env::remove_var("TELEGRAM_API_ID");
            env::remove_var("TELEGRAM_API_HASH");
            env::remove_var("TELEGRAM_SESSION_FILE");
        }
    }

    #[test]
    fn test_load_config_missing_id() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            env::remove_var("TELEGRAM_API_ID");
            env::set_var("TELEGRAM_API_HASH", "abcdef");
        }

        let err = AppConfig::load().unwrap_err();
        assert!(err.to_string().contains("TELEGRAM_API_ID is required"));

        unsafe {
            env::remove_var("TELEGRAM_API_HASH");
        }
    }

    #[test]
    fn test_load_config_invalid_id() {
        let _guard = ENV_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
        unsafe {
            env::set_var("TELEGRAM_API_ID", "not_an_int");
            env::set_var("TELEGRAM_API_HASH", "abcdef");
        }

        let err = AppConfig::load().unwrap_err();
        assert!(err.to_string().contains("must be a valid integer"));

        unsafe {
            env::remove_var("TELEGRAM_API_ID");
            env::remove_var("TELEGRAM_API_HASH");
        }
    }
}
