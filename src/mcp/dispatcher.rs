use crate::mcp::format;
use crate::mcp::protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
use crate::mcp::tool::{invalid_params, Tool};
use crate::mcp::tools::{registry, GROUPS};
use crate::telegram::TelegramService;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

const META_LIST: &str = "telegram_tools";
const META_CALL: &str = "telegram_call";

const INSTRUCTIONS: &str = "Telegram client. Results are compact text lines; ids come first. \
Find chats with list_chats(query=..., type=...) instead of listing everything. \
Message texts are cut at 300 chars; pass full=true to read them whole. \
Less common tools (admin, folders, contacts, pins, drafts, profile, bots) are not listed directly: \
call telegram_tools to see groups, telegram_tools(group) for their parameters, then telegram_call(tool, arguments).";

/// Which tools `tools/list` advertises. Every registered tool stays callable
/// by name either way.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ToolsMode {
    /// Core tools plus the two meta tools (default).
    Core,
    /// Every tool, flat.
    All,
}

impl ToolsMode {
    pub fn from_env() -> Self {
        match std::env::var("TELEGRAM_TOOLS").as_deref() {
            Ok("all") => ToolsMode::All,
            _ => ToolsMode::Core,
        }
    }
}

struct Entry {
    group: &'static str,
    tool: Box<dyn Tool>,
}

/// Static part of the server: tool metadata. Needs no Telegram connection, so
/// the stdio proxy can answer `initialize` / `tools/list` on its own.
pub struct Catalog {
    mode: ToolsMode,
    order: Vec<String>,
    tools: HashMap<String, Entry>,
}

impl Catalog {
    pub fn new(mode: ToolsMode) -> Self {
        let mut order = Vec::new();
        let mut tools = HashMap::new();
        for (group, tool) in registry() {
            let name = tool.info().name;
            order.push(name.clone());
            tools.insert(name, Entry { group, tool });
        }
        Self { mode, order, tools }
    }

    pub fn initialize_result(&self) -> Value {
        json!({
            "protocolVersion": "2024-11-05",
            "capabilities": { "tools": {} },
            "serverInfo": { "name": "telegram-mcp-rs", "version": env!("CARGO_PKG_VERSION") },
            "instructions": INSTRUCTIONS,
        })
    }

    fn schema(&self, name: &str) -> Value {
        let info = self.tools[name].tool.info();
        json!({ "name": info.name, "description": info.description, "inputSchema": info.input_schema })
    }

    pub fn tools_list_result(&self) -> Value {
        let mut list: Vec<Value> = self
            .order
            .iter()
            .filter(|n| self.mode == ToolsMode::All || self.tools[*n].group == "core")
            .map(|n| self.schema(n))
            .collect();
        if self.mode == ToolsMode::Core {
            let groups: Vec<&str> = GROUPS.iter().map(|(g, _)| *g).filter(|g| *g != "core").collect();
            list.push(json!({
                "name": META_LIST,
                "description": "List extra Telegram tool groups, or with `group` show that group's tools and parameters.",
                "inputSchema": { "type": "object", "properties": { "group": { "type": "string", "enum": groups } } }
            }));
            list.push(json!({
                "name": META_CALL,
                "description": "Call a Telegram tool from telegram_tools by name.",
                "inputSchema": {
                    "type": "object",
                    "properties": { "tool": { "type": "string" }, "arguments": { "type": "object" } },
                    "required": ["tool"]
                }
            }));
        }
        json!({ "tools": list })
    }

    fn describe(&self, group: Option<&str>) -> Value {
        let in_group = |g: &str| -> Vec<&String> { self.order.iter().filter(|n| self.tools[*n].group == g).collect() };
        let text = match group {
            None => GROUPS
                .iter()
                .filter(|(g, _)| *g != "core")
                .map(|(g, desc)| format!("{g}: {desc}\n  {}", in_group(g).iter().map(|n| n.as_str()).collect::<Vec<_>>().join(", ")))
                .collect::<Vec<_>>()
                .join("\n"),
            Some(g) if GROUPS.iter().any(|(name, _)| *name == g) => in_group(g)
                .into_iter()
                .map(|n| {
                    let info = self.tools[n].tool.info();
                    let props = info.input_schema.get("properties").cloned().unwrap_or(json!({}));
                    let required = info.input_schema.get("required").cloned().unwrap_or(json!([]));
                    format!("{}: {}\n  params: {} required: {}", info.name, info.description, props, required)
                })
                .collect::<Vec<_>>()
                .join("\n"),
            Some(g) => return format::err(format!("Unknown group '{g}'")),
        };
        format::ok(&text, false)
    }
}

pub struct Dispatcher {
    telegram: Arc<dyn TelegramService>,
    catalog: Arc<Catalog>,
}

impl Dispatcher {
    pub fn new(telegram: Arc<dyn TelegramService>) -> Self {
        Self::with_mode(telegram, ToolsMode::from_env())
    }

    pub fn with_mode(telegram: Arc<dyn TelegramService>, mode: ToolsMode) -> Self {
        Self { telegram, catalog: Arc::new(Catalog::new(mode)) }
    }

    pub async fn dispatch(&self, req: JsonRpcRequest) -> Option<JsonRpcResponse> {
        // Notifications get no response.
        let id = req.id?;

        let result = match req.method.as_str() {
            "initialize" => Ok(self.catalog.initialize_result()),
            "tools/list" => Ok(self.catalog.tools_list_result()),
            "tools/call" => self.handle_tools_call(req.params).await,
            "ping" => Ok(json!({})),
            _ => Err(JsonRpcError { code: -32601, message: format!("Method not found: {}", req.method), data: None }),
        };

        Some(match result {
            Ok(v) => JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: Some(v), error: None },
            Err(e) => JsonRpcResponse { jsonrpc: "2.0".to_string(), id, result: None, error: Some(e) },
        })
    }

    async fn handle_tools_call(&self, params: Option<Value>) -> Result<Value, JsonRpcError> {
        let params = params.ok_or_else(|| invalid_params("Missing params"))?;
        let name = params
            .get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| invalid_params("Missing or invalid 'name' in params"))?;
        let args = params.get("arguments").cloned().unwrap_or(json!({}));

        match name {
            META_LIST => Ok(self.catalog.describe(args.get("group").and_then(|g| g.as_str()))),
            META_CALL => {
                let tool = args.get("tool").and_then(|t| t.as_str()).ok_or_else(|| invalid_params("'tool' is required"))?;
                let inner = args.get("arguments").cloned().unwrap_or(json!({}));
                if tool == META_LIST || tool == META_CALL {
                    return Err(invalid_params("telegram_call cannot call meta tools"));
                }
                self.run_tool(tool, inner).await
            }
            _ => self.run_tool(name, args).await,
        }
    }

    async fn run_tool(&self, name: &str, args: Value) -> Result<Value, JsonRpcError> {
        if !self.catalog.tools.contains_key(name) {
            return Err(JsonRpcError {
                code: -32601,
                message: format!("Tool not found: {name}. See telegram_tools for available tools."),
                data: None,
            });
        }
        // Uploads and downloads can legitimately take a while.
        let secs = match name {
            "send_file" | "send_album" | "download_media" | "get_image" | "set_profile_photo" => 300,
            _ => 30,
        };
        let catalog = Arc::clone(&self.catalog);
        let telegram = Arc::clone(&self.telegram);
        let owned = name.to_string();
        // Run in its own task so a panic inside a tool becomes an error
        // response instead of taking the connection down.
        let task = tokio::spawn(async move { catalog.tools[&owned].tool.execute(args, telegram).await });
        match tokio::time::timeout(Duration::from_secs(secs), task).await {
            Ok(Ok(result)) => result,
            Ok(Err(join_err)) => Ok(format::err(format!("tool crashed: {join_err}"))),
            Err(_) => Err(JsonRpcError {
                code: -32000,
                message: format!("Tool execution timed out after {secs} seconds. Try a smaller limit."),
                data: None,
            }),
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
        async fn list_chats(&self, _limit: i32, _query: Option<&str>, _kind: Option<&str>) -> Result<Vec<ChatInfo>> {
            Ok(vec![ChatInfo { id: 456, title: "Test Chat".to_string(), username: None, kind: "group".to_string(), ..Default::default() }])
        }
        async fn get_chat_info(&self, _peer_id: i64) -> Result<ChatInfo> {
            Ok(ChatInfo { id: 456, title: "Test Chat".to_string(), username: None, kind: "group".to_string(), ..Default::default() })
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
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn get_history(&self, _peer_id: i64, _limit: i32, _offset_id: Option<i32>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn search_messages(&self, _peer_id: i64, _query: &str, _limit: i32, _from_user_id: Option<i64>, _from_username: Option<String>, _offset_id: Option<i32>, _min_date: Option<i32>, _max_date: Option<i32>, _filter: Option<String>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        
        async fn search_global_messages(&self, _query: &str, _limit: i32, _offset_id: Option<i32>, _min_date: Option<i32>, _max_date: Option<i32>, _filter: Option<String>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 1, sender_id: 123, text: "Hello".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn send_message(&self, _peer_id: i64, _text: &str, _reply_to_id: Option<i32>) -> Result<MessageInfo> {
            Ok(MessageInfo { id: 2, sender_id: 123, text: "Sent".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() })
        }
        async fn edit_message(&self, _peer_id: i64, _message_id: i32, _new_text: &str) -> Result<()> { Ok(()) }
        async fn mark_as_read(&self, _peer_id: i64) -> Result<()> { Ok(()) }
        async fn pin_message(&self, _peer_id: i64, _message_id: i32, _silent: bool) -> Result<()> { Ok(()) }
        async fn delete_messages(&self, _peer_id: i64, _message_ids: Vec<i32>, _revoke: bool) -> Result<()> { Ok(()) }
        async fn forward_messages(&self, _to_peer_id: i64, _from_peer_id: i64, _message_ids: Vec<i32>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 5, sender_id: 123, text: "Forwarded".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn send_reaction(&self, _peer_id: i64, _message_id: i32, _emoji: &str) -> Result<()> { Ok(()) }
        async fn get_scheduled_messages(&self, _peer_id: i64) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 6, sender_id: 123, text: "Scheduled".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn save_draft(&self, _peer_id: i64, _message: &str, _reply_to_id: Option<i32>) -> Result<()> { Ok(()) }
        async fn clear_drafts(&self) -> Result<()> { Ok(()) }
        async fn create_poll(&self, _peer_id: i64, _question: &str, _options: Vec<String>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Poll".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
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
        async fn wait_for_new_message(&self, _timeout: i32) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "msg".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
        async fn wait_for_settled_message(&self, _settle_ms: i32, _max_wait_ms: i32) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "msg".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
    }

    #[async_trait]
    impl MediaService for MockTelegramService {
        async fn send_file(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> {
            Ok(MessageInfo { id: 3, sender_id: 123, text: "File".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() })
        }
        async fn send_album(&self, _peer_id: i64, _paths: Vec<String>, _caption: Option<&str>) -> Result<Vec<MessageInfo>> {
            Ok(vec![MessageInfo { id: 4, sender_id: 123, text: "Album".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }])
        }
        async fn download_media(&self, _peer_id: i64, _message_id: i32, _path: Option<&str>) -> Result<PathBuf> {
            Ok(PathBuf::from("/tmp/test.media"))
        }
        async fn send_contact(&self, _peer_id: i64, _phone: &str, _first_name: &str, _last_name: &str) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Contact".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
        async fn send_gif(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Gif".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
        async fn send_sticker(&self, _peer_id: i64, _path: &str) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Sticker".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
        async fn send_voice(&self, _peer_id: i64, _path: &str, _caption: Option<&str>) -> Result<MessageInfo> { Ok(MessageInfo { id: 1, sender_id: 123, text: "Voice".to_string(), date: 0, reply_to_msg_id: None, ..Default::default() }) }
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

    fn call(name: &str, args: Value) -> JsonRpcRequest {
        JsonRpcRequest { jsonrpc: "2.0".to_string(), id: Some(Value::Number(9.into())), method: "tools/call".to_string(), params: Some(serde_json::json!({"name": name, "arguments": args})) }
    }

    #[test]
    fn core_mode_lists_core_plus_meta() {
        let core = Catalog::new(ToolsMode::Core).tools_list_result();
        let names: Vec<&str> = core["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(names.contains(&"list_chats") && names.contains(&META_LIST) && names.contains(&META_CALL));
        assert!(!names.contains(&"ban_user"));
        let all = Catalog::new(ToolsMode::All).tools_list_result();
        let all_names: Vec<&str> = all["tools"].as_array().unwrap().iter().map(|t| t["name"].as_str().unwrap()).collect();
        assert!(all_names.contains(&"ban_user") && !all_names.contains(&META_CALL));
    }

    #[tokio::test]
    async fn meta_tools_describe_and_call() {
        let dispatcher = Dispatcher::with_mode(Arc::new(MockTelegramService { authorized: true }), ToolsMode::Core);
        let groups = dispatcher.dispatch(call(META_LIST, serde_json::json!({}))).await.unwrap().result.unwrap();
        assert!(groups["content"][0]["text"].as_str().unwrap().contains("admin:"));
        let admin = dispatcher.dispatch(call(META_LIST, serde_json::json!({"group": "admin"}))).await.unwrap().result.unwrap();
        assert!(admin["content"][0]["text"].as_str().unwrap().contains("ban_user"));
        let via_call = dispatcher.dispatch(call(META_CALL, serde_json::json!({"tool": "get_bot_info", "arguments": {"bot_username": "x"}}))).await.unwrap();
        assert!(via_call.result.unwrap()["content"][0]["text"].as_str().unwrap().contains("About"));
        // Unlisted tools stay callable directly by name.
        let direct = dispatcher.dispatch(call("get_bot_info", serde_json::json!({"bot_username": "x"}))).await.unwrap();
        assert!(direct.error.is_none());
        let missing = dispatcher.dispatch(call("nope", serde_json::json!({}))).await.unwrap();
        assert!(missing.error.is_some());
    }
}
