use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct ListChatsTool;

#[async_trait]
impl Tool for ListChatsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_chats".to_string(),
            description: "List recent conversations.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "limit": { "type": "integer", "default": 10 }
                }
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(10) as i32;
        match telegram.list_chats(limit).await {
            Ok(chats) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&chats).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetChatInfoTool;

#[async_trait]
impl Tool for GetChatInfoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_chat_info".to_string(),
            description: "Get detailed information about a specific chat.".to_string(),
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
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        match telegram.get_chat_info(chat_id).await {
            Ok(chat) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&chat).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct CreateGroupTool;

#[async_trait]
impl Tool for CreateGroupTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "create_group".to_string(),
            description: "Create a new group chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "user_ids": { "type": "array", "items": { "type": "integer" } }
                },
                "required": ["title", "user_ids"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let title = match args.get("title").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return Err(invalid_params("title is required")),
        };
        let user_ids = match args.get("user_ids").and_then(|v| v.as_array()) {
            Some(ids) => ids.iter().filter_map(|v| v.as_i64()).collect::<Vec<_>>(),
            None => return Err(invalid_params("user_ids must be an array of integers")),
        };
        match telegram.create_group(title, user_ids).await {
            Ok(id) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Group created (ID: {})", id) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct CreateChannelTool;

#[async_trait]
impl Tool for CreateChannelTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "create_channel".to_string(),
            description: "Create a new channel or supergroup.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string" },
                    "about": { "type": "string" },
                    "megagroup": { "type": "boolean", "default": false }
                },
                "required": ["title", "about"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let title = match args.get("title").and_then(|v| v.as_str()) {
            Some(t) => t,
            None => return Err(invalid_params("title is required")),
        };
        let about = match args.get("about").and_then(|v| v.as_str()) {
            Some(a) => a,
            None => return Err(invalid_params("about is required")),
        };
        let megagroup = args.get("megagroup").and_then(|v| v.as_bool()).unwrap_or(false);
        match telegram.create_channel(title, about, megagroup).await {
            Ok(id) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Channel created (ID: {})", id) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct LeaveChatTool;

#[async_trait]
impl Tool for LeaveChatTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "leave_chat".to_string(),
            description: "Leave a group chat or channel.".to_string(),
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
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        match telegram.leave_chat(chat_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Left chat." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetChatMutedTool;

#[async_trait]
impl Tool for SetChatMutedTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_chat_muted".to_string(),
            description: "Mute or unmute a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "muted": { "type": "boolean" },
                    "until": { "type": "integer" }
                },
                "required": ["chat_id", "muted"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        let muted = match args.get("muted").and_then(|v| v.as_bool()) {
            Some(m) => m,
            None => return Err(invalid_params("muted is required")),
        };
        let until = args.get("until").and_then(|v| v.as_i64()).map(|v| v as i32);
        match telegram.set_chat_muted(chat_id, muted, until).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Chat {}", if muted { "muted" } else { "unmuted" }) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetChatArchivedTool;

#[async_trait]
impl Tool for SetChatArchivedTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_chat_archived".to_string(),
            description: "Archive or unarchive a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "archived": { "type": "boolean" }
                },
                "required": ["chat_id", "archived"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = match args.get("chat_id").and_then(|v| v.as_i64()) {
            Some(id) => id,
            None => return Err(invalid_params("chat_id is required")),
        };
        let archived = match args.get("archived").and_then(|v| v.as_bool()) {
            Some(a) => a,
            None => return Err(invalid_params("archived is required")),
        };
        match telegram.set_chat_archived(chat_id, archived).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Chat {}", if archived { "archived" } else { "unarchived" }) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct EditChatAboutTool;

#[async_trait]
impl Tool for EditChatAboutTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "edit_chat_about".to_string(),
            description: "Edit chat about text.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "about": { "type": "string" } }
                , "required": ["peer_id", "about"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let about = args.get("about").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("about is required"))?;
        match telegram.edit_chat_about(peer_id, about).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct EditChatPhotoTool;

#[async_trait]
impl Tool for EditChatPhotoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "edit_chat_photo".to_string(),
            description: "Edit chat photo.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "path": { "type": "string" } }
                , "required": ["peer_id", "path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        match telegram.edit_chat_photo(peer_id, path).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct EditChatTitleTool;

#[async_trait]
impl Tool for EditChatTitleTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "edit_chat_title".to_string(),
            description: "Edit chat title.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "title": { "type": "string" } }
                , "required": ["peer_id", "title"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let title = args.get("title").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("title is required"))?;
        match telegram.edit_chat_title(peer_id, title).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteChatPhotoTool;

#[async_trait]
impl Tool for DeleteChatPhotoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_chat_photo".to_string(),
            description: "Delete chat photo.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.delete_chat_photo(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
