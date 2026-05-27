use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct ListFoldersTool;

#[async_trait]
impl Tool for ListFoldersTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_folders".to_string(),
            description: "List Telegram chat folders (dialog filters).".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(&self, _args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        match telegram.list_folders().await {
            Ok(folders) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&folders).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetFolderTool;

#[async_trait]
impl Tool for GetFolderTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_folder".to_string(),
            description: "Get detailed information about a chat folder.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "folder_id": { "type": "integer" }
                },
                "required": ["folder_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let folder_id = match args.get("folder_id").and_then(|v| v.as_i64()) {
            Some(id) => id as i32,
            None => return Err(invalid_params("folder_id is required")),
        };
        match telegram.get_folder(folder_id).await {
            Ok(folder) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&folder).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct CreateFolderTool;

#[async_trait]
impl Tool for CreateFolderTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "create_folder".to_string(),
            description: "Create a chat folder.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "title": { "type": "string" }, "included_peers": { "type": "array" }, "excluded_peers": { "type": "array" } }
                , "required": ["title", "included_peers", "excluded_peers"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let title = args.get("title").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("title is required"))?;
        let included_peers = args.get("included_peers").and_then(|v| v.as_array()).ok_or_else(|| invalid_params("included_peers is required"))?.iter().filter_map(|v| v.as_i64()).collect::<Vec<_>>();
        let excluded_peers = args.get("excluded_peers").and_then(|v| v.as_array()).ok_or_else(|| invalid_params("excluded_peers is required"))?.iter().filter_map(|v| v.as_i64()).collect::<Vec<_>>();
        match telegram.create_folder(title, included_peers, excluded_peers).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteFolderTool;

#[async_trait]
impl Tool for DeleteFolderTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_folder".to_string(),
            description: "Delete a chat folder.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "folder_id": { "type": "integer" } }
                , "required": ["folder_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let folder_id = args.get("folder_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("folder_id is required"))? as i32;
        match telegram.delete_folder(folder_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct AddChatToFolderTool;

#[async_trait]
impl Tool for AddChatToFolderTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "add_chat_to_folder".to_string(),
            description: "Add a chat to a folder.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "folder_id": { "type": "integer" }, "peer_id": { "type": "integer" } }
                , "required": ["folder_id", "peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let folder_id = args.get("folder_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("folder_id is required"))? as i32;
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.add_chat_to_folder(folder_id, peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct RemoveChatFromFolderTool;

#[async_trait]
impl Tool for RemoveChatFromFolderTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "remove_chat_from_folder".to_string(),
            description: "Remove a chat from a folder.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "folder_id": { "type": "integer" }, "peer_id": { "type": "integer" } }
                , "required": ["folder_id", "peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let folder_id = args.get("folder_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("folder_id is required"))? as i32;
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.remove_chat_from_folder(folder_id, peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ReorderFoldersTool;

#[async_trait]
impl Tool for ReorderFoldersTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "reorder_folders".to_string(),
            description: "Reorder chat folders.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "folder_ids": { "type": "array" } }
                , "required": ["folder_ids"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let folder_ids = args.get("folder_ids").and_then(|v| v.as_array()).ok_or_else(|| invalid_params("folder_ids is required"))?.iter().filter_map(|v| v.as_i64().map(|id| id as i32)).collect::<Vec<_>>();
        match telegram.reorder_folders(folder_ids).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
