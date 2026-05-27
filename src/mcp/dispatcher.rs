use crate::mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::tool::Tool;
use crate::mcp::tools::register_all_tools;
use crate::telegram::TelegramService;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

pub struct Dispatcher {
    telegram: Arc<dyn TelegramService>,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Dispatcher {
    pub fn new(telegram: Arc<dyn TelegramService>) -> Self {
        Self { 
            telegram,
            tools: register_all_tools(),
        }
    }

    pub async fn dispatch(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        let id = match req.id {
            Some(id) => id,
            None => return None, // Notification
        };

        let result = match req.method.as_str() {
            "initialize" => Some(serde_json::json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "serverInfo": {
                    "name": "telegram-mcp-rs",
                    "version": "0.1.0"
                }
            })),
            "tools/list" => self.handle_tools_list().await,
            "tools/call" => {
                let (res, err) = self.handle_tools_call(req.params).await;
                if let Some(e) = err {
                    return Some(JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: None, error: Some(e) });
                }
                res
            }
            _ => {
                return Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32601,
                        message: format!("Method not found: {}", req.method),
                        data: None,
                    }),
                });
            }
        };

        Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result,
            error: None,
        })
    }

    async fn handle_tools_list(&self) -> Option<Value> {
        let mut tools_schema = Vec::new();
        for tool in self.tools.values() {
            let info = tool.info();
            tools_schema.push(serde_json::json!({
                "name": info.name,
                "description": info.description,
                "inputSchema": info.input_schema
            }));
        }

        Some(serde_json::json!({
            "tools": tools_schema
        }))
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> (Option<Value>, Option<JsonRpcError>) {
        let params = match params {
            Some(p) => p,
            None => return (None, Some(Self::invalid_params("Missing params"))),
        };

        let name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => return (None, Some(Self::invalid_params("Missing or invalid 'name' in params"))),
        };

        let args = params.get("arguments").cloned().unwrap_or(serde_json::json!({}));

        if let Some(tool) = self.tools.get(name) {
            return match tool.execute(args, Arc::clone(&self.telegram)).await {
                Ok(v) => (Some(v), None),
                Err(e) => (None, Some(e)),
            };
        }

        (None, Some(JsonRpcError { code: -32601, message: format!("Tool not found: {}", name), data: None }))
    }

    fn invalid_params(msg: &str) -> JsonRpcError {
        JsonRpcError {
            code: -32602,
            message: msg.to_string(),
            data: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::telegram::{UserInfo, ChatInfo, MessageInfo, ContactInfo, FolderInfo, BotInfo, TelegramService, AuthService, ChatService, MessageService, MediaService, ProfileService, BotService, ContactService, FolderService, AdminService, SearchService};
    use anyhow::Result;
    use async_trait::async_trait;
    use std::path::PathBuf;

    struct MockTelegramService { authorized: bool }

    #[async_trait]
    impl AuthService for MockTelegramService {
        async fn is_authorized(&self) -> Result<bool> { Ok(self.authorized) }
        async fn get_me(&self) -> Result<UserInfo> {
            if !self.authorized { anyhow::bail!("Not authorized") }
            Ok(UserInfo { id: 123, first_name: "Test".to_string(), last_name: None, username: Some("testuser".to_string()) })
        }
        async fn resolve_peer(&self, _peer_str: &str) -> Result<i64> { Ok(456) }
        async fn list_accounts(&self) -> Result<Vec<String>> { Ok(vec!["default".to_string()]) }
    }

    #[async_trait]
    impl ChatService for MockTelegramService {
        async fn list_chats(&self, _limit: i32) -> Result<Vec<ChatInfo>> {
            Ok(vec![ChatInfo { id: 456, title: "Test Chat".to_string(), username: None, chat_type: "Group".to_string() }])
        }
        async fn get_chat_info(&self, _peer_id: i64) -> Result<ChatInfo> {
            Ok(ChatInfo { id: 456, title: "Test Chat".to_string(), username: None, chat_type: "Group".to_string() })
        }
        async fn create_group(&self, _title: &str, _user_ids: Vec<i64>) -> Result<i64> { Ok(999) }
        async fn create_channel(&self, _title: &str, _about: &str, _megagroup: bool) -> Result<i64> { Ok(888) }
        async fn leave_chat(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn set_chat_muted(&self, _peer_id: i64, _muted: bool, _until: Option<i32>) -> Result<()> { Ok(()) }
        async fn set_chat_archived(&self, _peer_id: i64, _archived: bool) -> Result<()> { Ok(()) }
        async fn edit_chat_about(&self, _peer_id: i64, _about: &str) -> Result<()> { Ok(()) }
        async fn edit_chat_title(&self, _peer_id: i64, _title: &str) -> Result<()> { Ok(()) }
        async fn edit_chat_photo(&self, _peer_id: i64, _path: &str) -> Result<()> { Ok(()) }
        async fn delete_chat_photo(&self, _peer_id: i64) -> Result<()> { Ok(()) }
    }

    #[async_trait]
    impl MessageService for MockTelegramService {
        async fn get_messages(&self, _peer_id: i64, _limit: i32) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn get_history(&self, _peer_id: i64, _limit: i32, _offset_id: Option<i32>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn search_messages(&self, _peer_id: i64, _query: &str, _limit: i32) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn send_message(&self, _peer_id: i64, _text: &str, _reply_to_id: Option<i32>) -> Result<MessageInfo> {
            Ok(MessageInfo { id: 2, sender_id: 123, text: "Sent".to_string(), date: 0, reply_to_msg_id: None })
        }
        async fn edit_message(&self, _peer_id: i64, _message_id: i32, _new_text: &str) -> Result<()> { Ok(()) }
        async fn mark_as_read(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn pin_message(&self, _peer_id: i64, _message_id: i32, _silent: bool) -> Result<()> { Ok(()) }
        async fn delete_messages(&self, _peer_id: i64, _message_ids: Vec<i32>, _revoke: bool) -> Result<()> { Ok(()) }
        async fn forward_messages(&self, _to_peer_id: i64, _from_peer_id: i64, _message_ids: Vec<i32>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 5, sender_id: 123, text: "Forwarded".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn send_reaction(&self, _peer_id: i64, _message_id: i32, _emoji: &str) -> Result<()> { Ok(()) }
        async fn get_scheduled_messages(&self, _peer_id: i64) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 6, sender_id: 123, text: "Scheduled".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn save_draft(&self, _peer_id: i64, _message: &str, _reply_to_id: Option<i32>) -> Result<()> { Ok(()) }
        async fn clear_drafts(&self) -> Result<()> { Ok(()) }
        async fn create_poll(&self, _peer_id: i64, _question: &str, _options: Vec<String>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Poll".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn get_message_context(&self, _peer_id: i64, _message_id: i32) -> Result<Vec<MessageInfo>> { Ok(vec![]) }
        async fn get_message_link(&self, _peer_id: i64, _message_id: i32) -> Result<String> { Ok("link".to_string()) }
        async fn get_message_reactions(&self, _peer_id: i64, _message_id: i32) -> Result<Vec<String>> { Ok(vec![]) }
        async fn get_message_read_by(&self, _peer_id: i64, _message_id: i32) -> Result<Vec<UserInfo>> { Ok(vec![]) }
        async fn get_pinned_messages(&self, _peer_id: i64) -> Result<Vec<MessageInfo>> { Ok(vec![]) }
        async fn unpin_message(&self, _peer_id: i64, _message_id: i32) -> Result<()> { Ok(()) }
        async fn unpin_all_messages(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn delete_scheduled_message(&self, _peer_id: i64, _message_id: i32) -> Result<()> { Ok(()) }
        async fn remove_reaction(&self, _peer_id: i64, _message_id: i32) -> Result<()> { Ok(()) }
        async fn delete_chat_history(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn get_drafts(&self) -> Result<Vec<MessageInfo>> { Ok(vec![]) }
        async fn wait_for_new_message(&self, _timeout: i32) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "msg".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn wait_for_settled_message(&self, _settle_ms: i32, _max_wait_ms: i32) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "msg".to_string(), date: 0, reply_to_msg_id: None }) }
    }

    #[async_trait]
    impl MediaService for MockTelegramService {
        async fn send_file(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> {
            Ok(MessageInfo { id: 3, sender_id: 123, text: "File".to_string(), date: 0, reply_to_msg_id: None })
        }
        async fn send_album(&self, _peer_id: i64, _paths: Vec<String>, _caption: Option<&str>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 4, sender_id: 123, text: "Album".to_string(), date: 0, reply_to_msg_id: None }])
        }
        async fn download_media(&self, _peer_id: i64, _message_id: i32, _path: Option<&str>) -> Result<PathBuf> {
            Ok(PathBuf::from("/tmp/test.media"))
        }
        async fn send_contact(&self, _peer_id: i64, _phone: &str, _first_name: &str, _last_name: &str) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Contact".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn send_gif(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Gif".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn send_sticker(&self, _peer_id: i64, _path: &str) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Sticker".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn send_voice(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Voice".to_string(), date: 0, reply_to_msg_id: None }) }
        async fn get_gif_search(&self, _query: &str) -> Result<Vec<String>> { Ok(vec![]) }
        async fn get_media_info(&self, _peer_id: i64, _message_id: i32) -> Result<String> { Ok("info".to_string()) }
        async fn get_sticker_sets(&self) -> Result<Vec<String>> { Ok(vec![]) }
        async fn get_user_photos(&self, _user_id: i64) -> Result<Vec<String>> { Ok(vec![]) }
    }

    #[async_trait]
    impl ProfileService for MockTelegramService {
        async fn update_profile(&self, _first_name: Option<&str>, _last_name: Option<&str>, _about: Option<&str>) -> Result<()> { Ok(()) }
        async fn set_profile_photo(&self, _path: &str) -> Result<()> { Ok(()) }
        async fn delete_profile_photo(&self) -> Result<()> { Ok(()) }
        async fn get_user_status(&self, _user_id: i64) -> Result<String> { Ok("status".to_string()) }
        async fn resolve_username(&self, _username: &str) -> Result<i64> { Ok(123) }
    }

    #[async_trait]
    impl BotService for MockTelegramService {
        async fn get_bot_info(&self, _bot_username: &str) -> Result<BotInfo> {
            Ok(BotInfo { name: "Bot".to_string(), about: "About".to_string(), description: "Description".to_string() })
        }
        async fn list_inline_buttons(&self, _peer_id: i64, _message_id: i32) -> Result<Vec<String>> { Ok(vec![]) }
        async fn press_inline_button(&self, _peer_id: i64, _message_id: i32, _button_data: &str) -> Result<String> { Ok("Success".to_string()) }
        async fn set_bot_commands(&self, _commands: Vec<(String, String)>) -> Result<()> { Ok(()) }
    }

    #[async_trait]
    impl ContactService for MockTelegramService {
        async fn list_contacts(&self) -> Result<Vec<ContactInfo>> {
            Ok(vec![ContactInfo { user_id: 789, first_name: "Contact".to_string(), last_name: None, username: None, phone: None }])
        }
        async fn search_contacts(&self, _query: &str) -> Result<Vec<ContactInfo>> {
            Ok(vec![ContactInfo { user_id: 789, first_name: "Contact".to_string(), last_name: None, username: None, phone: None }])
        }
        async fn add_contact(&self, _user_id: i64, _first_name: &str, _last_name: &str, _phone: &str) -> Result<()> { Ok(()) }
        async fn delete_contact(&self, _user_id: i64) -> Result<()> { Ok(()) }
        async fn block_user(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn unblock_user(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn get_blocked_users(&self) -> Result<Vec<UserInfo>> { Ok(vec![]) }
        async fn import_contacts(&self, _contacts: Vec<(String, String, String)>) -> Result<()> { Ok(()) }
        async fn export_contacts(&self) -> Result<String> { Ok("export".to_string()) }
        async fn get_contact_ids(&self) -> Result<Vec<i64>> { Ok(vec![]) }
        async fn get_direct_chat_by_contact(&self, _contact_query: &str) -> Result<i64> { Ok(123) }
        async fn get_contact_chats(&self, _contact_id: i64) -> Result<Vec<ChatInfo>> { Ok(vec![]) }
        async fn get_last_interaction(&self, _contact_id: i64) -> Result<i32> { Ok(0) }
    }

    #[async_trait]
    impl FolderService for MockTelegramService {
        async fn list_folders(&self) -> Result<Vec<FolderInfo>> {
            Ok(vec![FolderInfo { id: 10, title: "Work".to_string(), emoticon: None, pinned_peers: vec![], included_peers: vec![], excluded_peers: vec![] }])
        }
        async fn get_folder(&self, _folder_id: i32) -> Result<FolderInfo> {
            Ok(FolderInfo { id: 10, title: "Work".to_string(), emoticon: None, pinned_peers: vec![], included_peers: vec![], excluded_peers: vec![] })
        }
        async fn create_folder(&self, _title: &str, _included_peers: Vec<i64>, _excluded_peers: Vec<i64>) -> Result<i32> { Ok(11) }
        async fn delete_folder(&self, _folder_id: i32) -> Result<()> { Ok(()) }
        async fn add_chat_to_folder(&self, _folder_id: i32, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn remove_chat_from_folder(&self, _folder_id: i32, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn reorder_folders(&self, _folder_ids: Vec<i32>) -> Result<()> { Ok(()) }
    }

    #[async_trait]
    impl SearchService for MockTelegramService {
        async fn search_global(&self, _query: &str) -> Result<Vec<MessageInfo>> { Ok(vec![]) }
        async fn search_public_chats(&self, _query: &str) -> Result<Vec<ChatInfo>> { Ok(vec![]) }
        async fn subscribe_public_channel(&self, _query: &str) -> Result<()> { Ok(()) }
        async fn get_common_chats(&self, _user_id: i64) -> Result<Vec<ChatInfo>> { Ok(vec![]) }
        async fn list_topics(&self, _peer_id: i64) -> Result<Vec<String>> { Ok(vec![]) }
        async fn get_privacy_settings(&self) -> Result<String> { Ok("privacy".to_string()) }
        async fn set_privacy_settings(&self, _key: &str, _value: &str) -> Result<()> { Ok(()) }
    }

    #[async_trait]
    impl AdminService for MockTelegramService {
        async fn get_participants(&self, _peer_id: i64, _limit: i32) -> Result<Vec<UserInfo>> {
            Ok(vec![UserInfo { id: 123, first_name: "Test".to_string(), last_name: None, username: None }])
        }
        async fn ban_user(&self, _peer_id: i64, _user_id: i64) -> Result<()> { Ok(()) }
        async fn promote_admin(&self, _peer_id: i64, _user_id: i64, _rank: Option<&str>) -> Result<()> { Ok(()) }
        async fn invite_users(&self, _peer_id: i64, _user_ids: Vec<i64>) -> Result<()> { Ok(()) }
        async fn demote_admin(&self, _peer_id: i64, _user_id: i64) -> Result<()> { Ok(()) }
        async fn edit_admin_rights(&self, _peer_id: i64, _user_id: i64, _add_admins: bool, _ban_users: bool, _pin_messages: bool) -> Result<()> { Ok(()) }
        async fn set_default_chat_permissions(&self, _peer_id: i64, _send_messages: bool, _send_media: bool) -> Result<()> { Ok(()) }
        async fn toggle_slow_mode(&self, _peer_id: i64, _seconds: i32) -> Result<()> { Ok(()) }
        async fn get_admins(&self, _peer_id: i64) -> Result<Vec<UserInfo>> { Ok(vec![]) }
        async fn get_banned_users(&self, _peer_id: i64) -> Result<Vec<UserInfo>> { Ok(vec![]) }
        async fn get_recent_actions(&self, _peer_id: i64, _limit: i32) -> Result<Vec<String>> { Ok(vec![]) }
        async fn unban_user(&self, _peer_id: i64, _user_id: i64) -> Result<()> { Ok(()) }
        async fn get_invite_link(&self, _peer_id: i64) -> Result<String> { Ok("link".to_string()) }
        async fn export_chat_invite(&self, _peer_id: i64) -> Result<String> { Ok("invite".to_string()) }
        async fn import_chat_invite(&self, _hash: &str) -> Result<i64> { Ok(1) }
        async fn join_chat_by_link(&self, _link: &str) -> Result<i64> { Ok(1) }
    }

    impl TelegramService for MockTelegramService {}

    #[tokio::test]
    async fn test_dispatch_initialize() {
        let service = Arc::new(MockTelegramService { authorized: true });
        let dispatcher = Dispatcher::new(service);
        let req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(Value::Number(1.into())), method: "initialize".to_string(), params: None };
        let resp = dispatcher.dispatch(req).await.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["serverInfo"]["name"], "telegram-mcp-rs");
    }

    #[tokio::test]
    async fn test_dispatch_get_me_authorized() {
        let service = Arc::new(MockTelegramService { authorized: true });
        let dispatcher = Dispatcher::new(service);
        let req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(Value::Number(2.into())), method: "tools/call".to_string(), params: Some(serde_json::json!({"name": "get_me", "arguments": {}})) };
        let resp = dispatcher.dispatch(req).await.unwrap();
        assert!(resp.error.is_none());
        assert_eq!(resp.result.unwrap()["content"][0]["text"], "User: Test (ID: 123)");
    }

    #[tokio::test]
    async fn test_dispatch_create_group() {
        let service = Arc::new(MockTelegramService { authorized: true });
        let dispatcher = Dispatcher::new(service);
        let req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(Value::Number(3.into())), method: "tools/call".to_string(), params: Some(serde_json::json!({"name": "create_group", "arguments": {"title": "New Group", "user_ids": [123]}})) };
        let resp = dispatcher.dispatch(req).await.unwrap();
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["content"][0]["text"].as_str().unwrap().contains("Group created"));
    }

    #[tokio::test]
    async fn test_dispatch_get_bot_info() {
        let service = Arc::new(MockTelegramService { authorized: true });
        let dispatcher = Dispatcher::new(service);
        let req = JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(Value::Number(4.into())), method: "tools/call".to_string(), params: Some(serde_json::json!({"name": "get_bot_info", "arguments": {"bot_username": "test_bot"}})) };
        let resp = dispatcher.dispatch(req).await.unwrap();
        assert!(resp.error.is_none());
        assert!(resp.result.unwrap()["content"][0]["text"].as_str().unwrap().contains("About"));
    }
}
