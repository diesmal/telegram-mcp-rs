use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct SearchGlobalTool;

#[async_trait]
impl Tool for SearchGlobalTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "search_global".to_string(),
            description: "Search for messages globally across all chats.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("query is required"))?;
        match telegram.search_global(query).await {
            Ok(msgs) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&msgs).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SearchPublicChatsTool;

#[async_trait]
impl Tool for SearchPublicChatsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "search_public_chats".to_string(),
            description: "Search for public chats and channels.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("query is required"))?;
        match telegram.search_public_chats(query).await {
            Ok(chats) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&chats).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SubscribePublicChannelTool;

#[async_trait]
impl Tool for SubscribePublicChannelTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "subscribe_public_channel".to_string(),
            description: "Subscribe to a public channel by its username.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" }
                },
                "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("query is required"))?;
        match telegram.subscribe_public_channel(query).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Subscribed successfully." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetCommonChatsTool;

#[async_trait]
impl Tool for GetCommonChatsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_common_chats".to_string(),
            description: "Get common chats with a user.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "user_id": { "type": "integer" }
                },
                "required": ["user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.get_common_chats(user_id).await {
            Ok(chats) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&chats).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ListTopicsTool;

#[async_trait]
impl Tool for ListTopicsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_topics".to_string(),
            description: "List forum topics in a supergroup.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "peer_id": { "type": "integer" }
                },
                "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.list_topics(peer_id).await {
            Ok(topics) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&topics).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetPrivacySettingsTool;

#[async_trait]
impl Tool for GetPrivacySettingsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_privacy_settings".to_string(),
            description: "Get Telegram privacy settings.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(&self, _args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        match telegram.get_privacy_settings().await {
            Ok(settings) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": settings }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetPrivacySettingsTool;

#[async_trait]
impl Tool for SetPrivacySettingsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_privacy_settings".to_string(),
            description: "Set Telegram privacy settings.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "value": { "type": "string" }
                },
                "required": ["key", "value"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let key = args.get("key").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("key is required"))?;
        let value = args.get("value").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("value is required"))?;
        match telegram.set_privacy_settings(key, value).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Privacy settings updated." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
