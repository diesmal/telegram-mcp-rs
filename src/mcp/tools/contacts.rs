use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct ListContactsTool;

#[async_trait]
impl Tool for ListContactsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "list_contacts".to_string(),
            description: "List your Telegram contacts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {}
            }),
        }
    }

    async fn execute(&self, _args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        match telegram.list_contacts().await {
            Ok(contacts) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&contacts).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SearchContactsTool;

#[async_trait]
impl Tool for SearchContactsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "search_contacts".to_string(),
            description: "Search your Telegram contacts.".to_string(),
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
        let query = match args.get("query").and_then(|v| v.as_str()) {
            Some(q) => q,
            None => return Err(invalid_params("query is required")),
        };
        match telegram.search_contacts(query).await {
            Ok(contacts) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&contacts).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct AddContactTool;

#[async_trait]
impl Tool for AddContactTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "add_contact".to_string(),
            description: "Add a user to your Telegram contacts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "user_id": { "type": "integer" },
                    "first_name": { "type": "string" },
                    "last_name": { "type": "string" },
                    "phone": { "type": "string" }
                },
                "required": ["user_id", "first_name"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        let first_name = args.get("first_name").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("first_name is required"))?;
        let last_name = args.get("last_name").and_then(|v| v.as_str()).unwrap_or("");
        let phone = args.get("phone").and_then(|v| v.as_str()).unwrap_or("");
        match telegram.add_contact(user_id, first_name, last_name, phone).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Contact added." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DeleteContactTool;

#[async_trait]
impl Tool for DeleteContactTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "delete_contact".to_string(),
            description: "Remove a user from your Telegram contacts.".to_string(),
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
        match telegram.delete_contact(user_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Contact deleted." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct BlockUserTool;

#[async_trait]
impl Tool for BlockUserTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "block_user".to_string(),
            description: "Block a user or peer.".to_string(),
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
        match telegram.block_user(peer_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "User blocked." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct UnblockUserTool;

#[async_trait]
impl Tool for UnblockUserTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "unblock_user".to_string(),
            description: "Unblock a user or peer.".to_string(),
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
        match telegram.unblock_user(peer_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "User unblocked." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetBlockedUsersTool;

#[async_trait]
impl Tool for GetBlockedUsersTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_blocked_users".to_string(),
            description: "Get blocked users.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.get_blocked_users().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ImportContactsTool;

#[async_trait]
impl Tool for ImportContactsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "import_contacts".to_string(),
            description: "Import contacts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "contacts": { "type": "array" } }
                , "required": ["contacts"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let contacts = vec![]; // Array extraction simplified for stub
        match telegram.import_contacts(contacts).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ExportContactsTool;

#[async_trait]
impl Tool for ExportContactsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "export_contacts".to_string(),
            description: "Export contacts.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.export_contacts().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetContactIdsTool;

#[async_trait]
impl Tool for GetContactIdsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_contact_ids".to_string(),
            description: "Get contact ids.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.get_contact_ids().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetDirectChatByContactTool;

#[async_trait]
impl Tool for GetDirectChatByContactTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_direct_chat_by_contact".to_string(),
            description: "Get direct chat by contact.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "contact_query": { "type": "string" } }
                , "required": ["contact_query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let contact_query = args.get("contact_query").and_then(|v| v.as_str()).unwrap_or("").to_string();
        match telegram.get_direct_chat_by_contact(&contact_query).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetContactChatsTool;

#[async_trait]
impl Tool for GetContactChatsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_contact_chats".to_string(),
            description: "Get contact chats.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "contact_id": { "type": "integer" } }
                , "required": ["contact_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let contact_id = args.get("contact_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("contact_id is required"))?;
        match telegram.get_contact_chats(contact_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetLastInteractionTool;

#[async_trait]
impl Tool for GetLastInteractionTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_last_interaction".to_string(),
            description: "Get last interaction.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "contact_id": { "type": "integer" } }
                , "required": ["contact_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let contact_id = args.get("contact_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("contact_id is required"))?;
        match telegram.get_last_interaction(contact_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
