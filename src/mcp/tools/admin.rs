use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct BanUserTool;

#[async_trait]
impl Tool for BanUserTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "ban_user".to_string(),
            description: "Ban a user from a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "user_id": { "type": "integer" }
                },
                "required": ["chat_id", "user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.ban_user(chat_id, user_id).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "User banned." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct PromoteAdminTool;

#[async_trait]
impl Tool for PromoteAdminTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "promote_admin".to_string(),
            description: "Promote a user to admin.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "user_id": { "type": "integer" },
                    "rank": { "type": "string" }
                },
                "required": ["chat_id", "user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        let rank = args.get("rank").and_then(|v| v.as_str());
        match telegram.promote_admin(chat_id, user_id, rank).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "User promoted." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetParticipantsTool;

#[async_trait]
impl Tool for GetParticipantsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_participants".to_string(),
            description: "Get list of chat participants.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "limit": { "type": "integer", "default": 100 }
                },
                "required": ["chat_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(100) as i32;
        match telegram.get_participants(chat_id, limit).await {
            Ok(users) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&users).unwrap() }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct InviteUsersTool;

#[async_trait]
impl Tool for InviteUsersTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "invite_users".to_string(),
            description: "Invite users to a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "user_ids": { "type": "array", "items": { "type": "integer" } }
                },
                "required": ["chat_id", "user_ids"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let user_ids = args.get("user_ids").and_then(|v| v.as_array())
            .ok_or_else(|| invalid_params("user_ids must be an array of integers"))?
            .iter().filter_map(|v| v.as_i64()).collect::<Vec<_>>();
        match telegram.invite_users(chat_id, user_ids).await {
            Ok(_) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": "Users invited." }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DemoteAdminTool;

#[async_trait]
impl Tool for DemoteAdminTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "demote_admin".to_string(),
            description: "Demote an admin.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "user_id": { "type": "integer" } }
                , "required": ["peer_id", "user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.demote_admin(peer_id, user_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct EditAdminRightsTool;

#[async_trait]
impl Tool for EditAdminRightsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "edit_admin_rights".to_string(),
            description: "Edit admin rights.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "user_id": { "type": "integer" }, "add_admins": { "type": "boolean" }, "ban_users": { "type": "boolean" }, "pin_messages": { "type": "boolean" } }
                , "required": ["peer_id", "user_id", "add_admins", "ban_users", "pin_messages"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        let add_admins = args.get("add_admins").and_then(|v| v.as_bool()).ok_or_else(|| invalid_params("add_admins is required"))?;
        let ban_users = args.get("ban_users").and_then(|v| v.as_bool()).ok_or_else(|| invalid_params("ban_users is required"))?;
        let pin_messages = args.get("pin_messages").and_then(|v| v.as_bool()).ok_or_else(|| invalid_params("pin_messages is required"))?;
        match telegram.edit_admin_rights(peer_id, user_id, add_admins, ban_users, pin_messages).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SetDefaultChatPermissionsTool;

#[async_trait]
impl Tool for SetDefaultChatPermissionsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "set_default_chat_permissions".to_string(),
            description: "Set default chat permissions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "send_messages": { "type": "boolean" }, "send_media": { "type": "boolean" } }
                , "required": ["peer_id", "send_messages", "send_media"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let send_messages = args.get("send_messages").and_then(|v| v.as_bool()).ok_or_else(|| invalid_params("send_messages is required"))?;
        let send_media = args.get("send_media").and_then(|v| v.as_bool()).ok_or_else(|| invalid_params("send_media is required"))?;
        match telegram.set_default_chat_permissions(peer_id, send_messages, send_media).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ToggleSlowModeTool;

#[async_trait]
impl Tool for ToggleSlowModeTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "toggle_slow_mode".to_string(),
            description: "Toggle slow mode.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "seconds": { "type": "integer" } }
                , "required": ["peer_id", "seconds"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let seconds = args.get("seconds").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("seconds is required"))? as i32;
        match telegram.toggle_slow_mode(peer_id, seconds).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetAdminsTool;

#[async_trait]
impl Tool for GetAdminsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_admins".to_string(),
            description: "Get chat admins.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.get_admins(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetBannedUsersTool;

#[async_trait]
impl Tool for GetBannedUsersTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_banned_users".to_string(),
            description: "Get banned users.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.get_banned_users(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetRecentActionsTool;

#[async_trait]
impl Tool for GetRecentActionsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_recent_actions".to_string(),
            description: "Get recent admin actions.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "limit": { "type": "integer" } }
                , "required": ["peer_id", "limit"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let limit = args.get("limit").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("limit is required"))? as i32;
        match telegram.get_recent_actions(peer_id, limit).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct UnbanUserTool;

#[async_trait]
impl Tool for UnbanUserTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "unban_user".to_string(),
            description: "Unban a user.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "user_id": { "type": "integer" } }
                , "required": ["peer_id", "user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.unban_user(peer_id, user_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetInviteLinkTool;

#[async_trait]
impl Tool for GetInviteLinkTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_invite_link".to_string(),
            description: "Get invite link.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.get_invite_link(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ExportChatInviteTool;

#[async_trait]
impl Tool for ExportChatInviteTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "export_chat_invite".to_string(),
            description: "Export chat invite.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" } }
                , "required": ["peer_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        match telegram.export_chat_invite(peer_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct ImportChatInviteTool;

#[async_trait]
impl Tool for ImportChatInviteTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "import_chat_invite".to_string(),
            description: "Import chat invite.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "hash": { "type": "string" } }
                , "required": ["hash"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let hash = args.get("hash").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("hash is required"))?;
        match telegram.import_chat_invite(hash).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct JoinChatByLinkTool;

#[async_trait]
impl Tool for JoinChatByLinkTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "join_chat_by_link".to_string(),
            description: "Join chat by link.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "link": { "type": "string" } }
                , "required": ["link"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let link = args.get("link").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("link is required"))?;
        match telegram.join_chat_by_link(link).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
