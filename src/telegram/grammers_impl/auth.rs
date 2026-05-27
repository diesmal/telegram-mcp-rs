use crate::telegram::{AuthService, UserInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;

#[async_trait]
impl AuthService for GrammersService {
    async fn is_authorized(&self) -> Result<bool> {
        Ok(self.client.is_authorized().await?)
    }

    async fn get_me(&self) -> Result<UserInfo> {
        let user = self.client.get_me().await?;
        Ok(UserInfo {
            id: self.to_i64(user.id()),
            first_name: user.first_name().unwrap_or_default().to_string(),
            last_name: user.last_name().map(|s| s.to_string()),
            username: user.username().map(|s| s.to_string()),
        })
    }

    async fn resolve_peer(&self, peer_str: &str) -> Result<i64> {
        if let Ok(id) = peer_str.parse::<i64>() {
            return Ok(id);
        }
        let peer = self.client.resolve_username(peer_str).await?.context("Peer not found")?;
        Ok(peer.id().bot_api_dialog_id())
    }

    async fn list_accounts(&self) -> Result<Vec<String>> {
        Ok(vec!["default".to_string()])
    }
}
