use crate::mcp::protocol::JsonRpcError;
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct ToolInfo {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn info(&self) -> ToolInfo;
    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError>;
}

pub fn invalid_params(msg: &str) -> JsonRpcError {
    JsonRpcError {
        code: -32602,
        message: msg.to_string(),
        data: None,
    }
}
