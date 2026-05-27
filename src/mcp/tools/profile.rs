use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct GetMeTool;

#[async_trait]
impl Tool for GetMeTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_me".to_string(),
            description: "Get your own user information.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {},
                "required": []
            }),
        }
    }

    async fn execute(&self, _args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        match telegram.get_me().await {
            Ok(user) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("User: {} (ID: {})", user.first_name, user.id) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct UpdateProfileTool;

#[async_trait]
impl Tool for UpdateProfileTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "update_profile".to_string(),
            description: "Update your profile information.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "first_name": { "type": "string" },
                    "last_name": { "type": "string" },
                    "about": { "type": "string" }
                }
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let first_name = args.get("first_name").and_then(|v| v.as_str());
        let last_name = args.get("last_name").and_then(|v| v.as_str());
        let about = args.get("about").and_then(|v| v.as_str());
        match telegram.update_profile(first_name, last_name, about).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Profile updated." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetProfilePhotoTool;

#[async_trait]
impl Tool for SetProfilePhotoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_profile_photo".to_string(),
            description: "Set profile photo.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        match telegram.set_profile_photo(path).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Profile photo set." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteProfilePhotoTool;

#[async_trait]
impl Tool for DeleteProfilePhotoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_profile_photo".to_string(),
            description: "Delete profile photo.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.delete_profile_photo().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetUserStatusTool;

#[async_trait]
impl Tool for GetUserStatusTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_user_status".to_string(),
            description: "Get user status.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "user_id": { "type": "integer" } }
                , "required": ["user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.get_user_status(user_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ResolveUsernameTool;

#[async_trait]
impl Tool for ResolveUsernameTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "resolve_username".to_string(),
            description: "Resolve username.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "username": { "type": "string" } }
                , "required": ["username"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let username = args.get("username").and_then(|v| v.as_str()).unwrap_or("").to_string();
        match telegram.resolve_username(&username).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ListAccountsTool;

#[async_trait]
impl Tool for ListAccountsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_accounts".to_string(),
            description: "List accounts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.list_accounts().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
