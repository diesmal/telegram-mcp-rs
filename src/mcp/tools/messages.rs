use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct GetMessagesTool;

#[async_trait]
impl Tool for GetMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_messages".to_string(),
            description: "Get messages from a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "limit": { "type": "integer", "default": 20 }
                },
                "required": ["chat_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i32;
        match telegram.get_messages(chat_id, limit).await {
            Ok(msgs) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&msgs).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetHistoryTool;

#[async_trait]
impl Tool for GetHistoryTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_history".to_string(),
            description: "Get paginated message history.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "limit": { "type": "integer", "default": 20 },
                    "offset_id": { "type": "integer" }
                },
                "required": ["chat_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i32;
        let offset_id = args.get("offset_id").and_then(|v| v.as_i64()).map(|v| v as i32);
        match telegram.get_history(chat_id, limit, offset_id).await {
            Ok(msgs) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&msgs).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SearchMessagesTool;

#[async_trait]
impl Tool for SearchMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "search_messages".to_string(),
            description: "Search for messages within a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "query": { "type": "string" },
                    "limit": { "type": "integer", "default": 20 }
                },
                "required": ["chat_id", "query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("query is required"))?;
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(20) as i32;
        match telegram.search_messages(chat_id, query, limit).await {
            Ok(msgs) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&msgs).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendMessageTool;

#[async_trait]
impl Tool for SendMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_message".to_string(),
            description: "Send a text message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "text": { "type": "string" },
                    "reply_to_id": { "type": "integer" }
                },
                "required": ["chat_id", "text"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let text = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("text is required"))?;
        let reply_to_id = args.get("reply_to_id").and_then(|v| v.as_i64()).map(|v| v as i32);
        match telegram.send_message(chat_id, text, reply_to_id).await {
            Ok(msg) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Message sent (ID: {})", msg.id) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct EditMessageTool;

#[async_trait]
impl Tool for EditMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "edit_message".to_string(),
            description: "Edit an existing message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" },
                    "text": { "type": "string" }
                },
                "required": ["chat_id", "message_id", "text"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        let text = args.get("text").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("text is required"))?;
        match telegram.edit_message(chat_id, message_id, text).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Message edited." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteMessagesTool;

#[async_trait]
impl Tool for DeleteMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_messages".to_string(),
            description: "Delete messages from a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_ids": { "type": "array", "items": { "type": "integer" } },
                    "revoke": { "type": "boolean", "default": true }
                },
                "required": ["chat_id", "message_ids"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message_ids = args.get("message_ids").and_then(|v| v.as_array())
            .ok_or_else(|| invalid_params("message_ids must be an array of integers"))?
            .iter().filter_map(|v| v.as_i64().map(|id| id as i32)).collect::<Vec<_>>();
        let revoke = args.get("revoke").and_then(|v| v.as_bool()).unwrap_or(true);
        match telegram.delete_messages(chat_id, message_ids, revoke).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Messages deleted." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ForwardMessagesTool;

#[async_trait]
impl Tool for ForwardMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "forward_messages".to_string(),
            description: "Forward messages from one chat to another.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "to_chat_id": { "type": "integer" },
                    "from_chat_id": { "type": "integer" },
                    "message_ids": { "type": "array", "items": { "type": "integer" } }
                },
                "required": ["to_chat_id", "from_chat_id", "message_ids"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let to_chat_id = args.get("to_chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("to_chat_id is required"))?;
        let from_chat_id = args.get("from_chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("from_chat_id is required"))?;
        let message_ids = args.get("message_ids").and_then(|v| v.as_array())
            .ok_or_else(|| invalid_params("message_ids must be an array of integers"))?
            .iter().filter_map(|v| v.as_i64().map(|id| id as i32)).collect::<Vec<_>>();
        match telegram.forward_messages(to_chat_id, from_chat_id, message_ids).await {
            Ok(msgs) => {
                let ids: Vec<i32> = msgs.iter().map(|m| m.id).collect();
                Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Messages forwarded (IDs: {:?})", ids) }] }))
            }
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendReactionTool;

#[async_trait]
impl Tool for SendReactionTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_reaction".to_string(),
            description: "Send an emoji reaction to a message. Pass empty string to remove.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" },
                    "emoji": { "type": "string" }
                },
                "required": ["chat_id", "message_id", "emoji"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        let emoji = args.get("emoji").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("emoji is required"))?;
        match telegram.send_reaction(chat_id, message_id, emoji).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Reaction sent." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetScheduledMessagesTool;

#[async_trait]
impl Tool for GetScheduledMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_scheduled_messages".to_string(),
            description: "List scheduled messages for a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" }
                },
                "required": ["chat_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        match telegram.get_scheduled_messages(chat_id).await {
            Ok(msgs) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&msgs).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SaveDraftTool;

#[async_trait]
impl Tool for SaveDraftTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "save_draft".to_string(),
            description: "Save a draft message in a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message": { "type": "string" },
                    "reply_to_id": { "type": "integer" }
                },
                "required": ["chat_id", "message"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message = args.get("message").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("message is required"))?;
        let reply_to_id = args.get("reply_to_id").and_then(|v| v.as_i64()).map(|v| v as i32);
        match telegram.save_draft(chat_id, message, reply_to_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Draft saved." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ClearDraftsTool;

#[async_trait]
impl Tool for ClearDraftsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "clear_drafts".to_string(),
            description: "Clear all drafts on the account.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(&self, _args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        match telegram.clear_drafts().await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "All drafts cleared." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct CreatePollTool;

#[async_trait]
impl Tool for CreatePollTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "create_poll".to_string(),
            description: "Create a poll.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "question": { "type": "string" }, "options": { "type": "array" } }
                , "required": ["peer_id", "question", "options"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let question = args.get("question").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("question is required"))?;
        let options = args.get("options").and_then(|v| v.as_array()).ok_or_else(|| invalid_params("options is required"))?.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>();
        match telegram.create_poll(peer_id, question, options).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetMessageContextTool;

#[async_trait]
impl Tool for GetMessageContextTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_message_context".to_string(),
            description: "Get message context.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.get_message_context(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetMessageLinkTool;

#[async_trait]
impl Tool for GetMessageLinkTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_message_link".to_string(),
            description: "Get message link.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.get_message_link(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetMessageReactionsTool;

#[async_trait]
impl Tool for GetMessageReactionsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_message_reactions".to_string(),
            description: "Get message reactions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.get_message_reactions(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetMessageReadByTool;

#[async_trait]
impl Tool for GetMessageReadByTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_message_read_by".to_string(),
            description: "Get message read by.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.get_message_read_by(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetPinnedMessagesTool;

#[async_trait]
impl Tool for GetPinnedMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_pinned_messages".to_string(),
            description: "Get pinned messages.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.get_pinned_messages(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct UnpinMessageTool;

#[async_trait]
impl Tool for UnpinMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "unpin_message".to_string(),
            description: "Unpin message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.unpin_message(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct UnpinAllMessagesTool;

#[async_trait]
impl Tool for UnpinAllMessagesTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "unpin_all_messages".to_string(),
            description: "Unpin all messages.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.unpin_all_messages(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteScheduledMessageTool;

#[async_trait]
impl Tool for DeleteScheduledMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_scheduled_message".to_string(),
            description: "Delete scheduled message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.delete_scheduled_message(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct RemoveReactionTool;

#[async_trait]
impl Tool for RemoveReactionTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "remove_reaction".to_string(),
            description: "Remove reaction.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.remove_reaction(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteChatHistoryTool;

#[async_trait]
impl Tool for DeleteChatHistoryTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_chat_history".to_string(),
            description: "Delete chat history.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.delete_chat_history(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct PinMessageTool;

#[async_trait]
impl Tool for PinMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "pin_message".to_string(),
            description: "Pin a message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" }, "silent": { "type": "boolean" } }
                , "required": ["peer_id", "message_id", "silent"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let silent = args.get("silent").and_then(|v| v.as_bool()).unwrap_or(false);
        match telegram.pin_message(peer_id, message_id, silent).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct MarkAsReadTool;

#[async_trait]
impl Tool for MarkAsReadTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "mark_as_read".to_string(),
            description: "Mark chat as read.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.mark_as_read(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetDraftsTool;

#[async_trait]
impl Tool for GetDraftsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_drafts".to_string(),
            description: "Get all drafts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.get_drafts().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct WaitForNewMessageTool;

#[async_trait]
impl Tool for WaitForNewMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "wait_for_new_message".to_string(),
            description: "Wait for new message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "timeout": { "type": "integer" } }
                , "required": ["timeout"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let timeout = args.get("timeout").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        match telegram.wait_for_new_message(timeout).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct WaitForSettledMessageTool;

#[async_trait]
impl Tool for WaitForSettledMessageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "wait_for_settled_message".to_string(),
            description: "Wait for settled message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "settle_ms": { "type": "integer" }, "max_wait_ms": { "type": "integer" } }
                , "required": ["settle_ms", "max_wait_ms"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let settle_ms = args.get("settle_ms").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        let max_wait_ms = args.get("max_wait_ms").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
        match telegram.wait_for_settled_message(settle_ms, max_wait_ms).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
