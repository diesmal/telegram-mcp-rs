use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct GetBotInfoTool;

#[async_trait]
impl Tool for GetBotInfoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_bot_info".to_string(),
            description: "Get information about a bot.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "bot_username": { "type": "string" }
                },
                "required": ["bot_username"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let bot_username = match args.get("bot_username").and_then(|v| v.as_str()) {
            Some(u) => u,
            None => return Err(invalid_params("bot_username is required")),
        };
        match telegram.get_bot_info(bot_username).await {
            Ok(info) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&info).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ListInlineButtonsTool;

#[async_trait]
impl Tool for ListInlineButtonsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_inline_buttons".to_string(),
            description: "List inline buttons attached to a specific message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" }
                },
                "required": ["chat_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        let message_id = match args.get("message_id").and_then(|v| v.as_i64()) {
            Some(id) => id as i32,
            None => return Err(invalid_params("message_id is required")),
        };
        match telegram.list_inline_buttons(chat_id, message_id).await {
            Ok(buttons) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&buttons).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct PressInlineButtonTool;

#[async_trait]
impl Tool for PressInlineButtonTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "press_inline_button".to_string(),
            description: "Press an inline button attached to a message using its callback data.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" },
                    "button_data": { "type": "string", "description": "The base64 encoded data from list_inline_buttons" }
                },
                "required": ["chat_id", "message_id", "button_data"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        let message_id = match args.get("message_id").and_then(|v| v.as_i64()) {
            Some(id) => id as i32,
            None => return Err(invalid_params("message_id is required")),
        };
        let button_data = match args.get("button_data").and_then(|v| v.as_str()) {
            Some(d) => d,
            None => return Err(invalid_params("button_data is required")),
        };
        match telegram.press_inline_button(chat_id, message_id, button_data).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": res }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetBotCommandsTool;

#[async_trait]
impl Tool for SetBotCommandsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_bot_commands".to_string(),
            description: "Set bot commands.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "commands": { "type": "array" } }
                , "required": ["commands"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let commands = vec![]; // Array extraction simplified for stub
        match telegram.set_bot_commands(commands).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
