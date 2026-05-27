use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatInfo {
    pub id: i64,
    pub title: String,
    pub username: Option<String>,
    pub chat_type: String, // "User", "Group", "Channel", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInfo {
    pub id: i32,
    pub sender_id: i64,
    pub text: String,
    pub date: i64,
    pub reply_to_msg_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactInfo {
    pub user_id: i64,
    pub first_name: String,
    pub last_name: Option<String>,
    pub username: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderInfo {
    pub id: i32,
    pub title: String,
    pub emoticon: Option<String>,
    pub pinned_peers: Vec<i64>,
    pub included_peers: Vec<i64>,
    pub excluded_peers: Vec<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotInfo {
    pub name: String,
    pub about: String,
    pub description: String,
}

#[async_trait]
pub trait TelegramService: 
    AuthService + ChatService + MessageService + MediaService + ProfileService + BotService + ContactService + FolderService + AdminService + SearchService
    + Send + Sync {}

#[async_trait]
pub trait AuthService {
    async fn is_authorized(&self) -> Result<bool>;
    async fn get_me(&self) -> Result<UserInfo>;
    async fn resolve_peer(&self, peer_str: &str) -> Result<i64>;
    async fn list_accounts(&self) -> Result<Vec<String>>;
}

#[async_trait]
pub trait ChatService {
    async fn list_chats(&self, limit: i32) -> Result<Vec<ChatInfo>>;
    async fn get_chat_info(&self, peer_id: i64) -> Result<ChatInfo>;
    async fn create_group(&self, title: &str, user_ids: Vec<i64>) -> Result<i64>;
    async fn create_channel(&self, title: &str, about: &str, megagroup: bool) -> Result<i64>;
    async fn leave_chat(&self, peer_id: i64) -> Result<()>;
    async fn set_chat_muted(&self, peer_id: i64, muted: bool, until: Option<i32>) -> Result<()>;
    async fn set_chat_archived(&self, peer_id: i64, archived: bool) -> Result<()>;
    async fn edit_chat_about(&self, peer_id: i64, about: &str) -> Result<()>;
    async fn edit_chat_title(&self, peer_id: i64, title: &str) -> Result<()>;
    async fn edit_chat_photo(&self, peer_id: i64, path: &str) -> Result<()>;
    async fn delete_chat_photo(&self, peer_id: i64) -> Result<()>;
}

#[async_trait]
pub trait MessageService {
    async fn get_messages(&self, peer_id: i64, limit: i32) -> Result<Vec<MessageInfo>>;
    async fn get_history(&self, peer_id: i64, limit: i32, offset_id: Option<i32>) -> Result<Vec<MessageInfo>>;
    async fn search_messages(&self, peer_id: i64, query: &str, limit: i32) -> Result<Vec<MessageInfo>>;
    async fn send_message(&self, peer_id: i64, text: &str, reply_to_id: Option<i32>) -> Result<MessageInfo>;
    async fn edit_message(&self, peer_id: i64, message_id: i32, new_text: &str) -> Result<()>;
    async fn mark_as_read(&self, peer_id: i64) -> Result<()>;
    async fn pin_message(&self, peer_id: i64, message_id: i32, silent: bool) -> Result<()>;
    async fn delete_messages(&self, peer_id: i64, message_ids: Vec<i32>, revoke: bool) -> Result<()>;
    async fn forward_messages(&self, to_peer_id: i64, from_peer_id: i64, message_ids: Vec<i32>) -> Result<Vec<MessageInfo>>;
    async fn send_reaction(&self, peer_id: i64, message_id: i32, emoji: &str) -> Result<()>;
    async fn get_scheduled_messages(&self, peer_id: i64) -> Result<Vec<MessageInfo>>;
    async fn save_draft(&self, peer_id: i64, message: &str, reply_to_id: Option<i32>) -> Result<()>;
    async fn clear_drafts(&self) -> Result<()>;
    async fn create_poll(&self, peer_id: i64, question: &str, options: Vec<String>) -> Result<MessageInfo>;
    async fn get_message_context(&self, peer_id: i64, message_id: i32) -> Result<Vec<MessageInfo>>;
    async fn get_message_link(&self, peer_id: i64, message_id: i32) -> Result<String>;
    async fn get_message_reactions(&self, peer_id: i64, message_id: i32) -> Result<Vec<String>>;
    async fn get_message_read_by(&self, peer_id: i64, message_id: i32) -> Result<Vec<UserInfo>>;
    async fn get_pinned_messages(&self, peer_id: i64) -> Result<Vec<MessageInfo>>;
    async fn unpin_message(&self, peer_id: i64, message_id: i32) -> Result<()>;
    async fn unpin_all_messages(&self, peer_id: i64) -> Result<()>;
    async fn delete_scheduled_message(&self, peer_id: i64, message_id: i32) -> Result<()>;
    async fn remove_reaction(&self, peer_id: i64, message_id: i32) -> Result<()>;
    async fn delete_chat_history(&self, peer_id: i64) -> Result<()>;
    async fn get_drafts(&self) -> Result<Vec<MessageInfo>>;
    async fn wait_for_new_message(&self, timeout: i32) -> Result<MessageInfo>;
    async fn wait_for_settled_message(&self, settle_ms: i32, max_wait_ms: i32) -> Result<MessageInfo>;
}

#[async_trait]
pub trait MediaService {
    async fn send_file(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo>;
    async fn send_album(&self, peer_id: i64, paths: Vec<String>, caption: Option<&str>) -> Result<Vec<MessageInfo>>;
    async fn download_media(&self, peer_id: i64, message_id: i32, path: Option<&str>) -> Result<PathBuf>;
    async fn send_contact(&self, peer_id: i64, phone: &str, first_name: &str, last_name: &str) -> Result<MessageInfo>;
    async fn send_gif(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo>;
    async fn send_sticker(&self, peer_id: i64, path: &str) -> Result<MessageInfo>;
    async fn send_voice(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo>;
    async fn get_gif_search(&self, query: &str) -> Result<Vec<String>>;
    async fn get_media_info(&self, peer_id: i64, message_id: i32) -> Result<String>;
    async fn get_sticker_sets(&self) -> Result<Vec<String>>;
    async fn get_user_photos(&self, user_id: i64) -> Result<Vec<String>>;
}

#[async_trait]
pub trait ProfileService {
    async fn update_profile(&self, first_name: Option<&str>, last_name: Option<&str>, about: Option<&str>) -> Result<()>;
    async fn set_profile_photo(&self, path: &str) -> Result<()>;
    async fn delete_profile_photo(&self) -> Result<()>;
    async fn get_user_status(&self, user_id: i64) -> Result<String>;
    async fn resolve_username(&self, username: &str) -> Result<i64>;
}

#[async_trait]
pub trait BotService {
    async fn get_bot_info(&self, bot_username: &str) -> Result<BotInfo>;
    async fn list_inline_buttons(&self, peer_id: i64, message_id: i32) -> Result<Vec<String>>;
    async fn press_inline_button(&self, peer_id: i64, message_id: i32, button_data: &str) -> Result<String>;
    async fn set_bot_commands(&self, commands: Vec<(String, String)>) -> Result<()>;
}

#[async_trait]
pub trait ContactService {
    async fn list_contacts(&self) -> Result<Vec<ContactInfo>>;
    async fn search_contacts(&self, query: &str) -> Result<Vec<ContactInfo>>;
    async fn add_contact(&self, user_id: i64, first_name: &str, last_name: &str, phone: &str) -> Result<()>;
    async fn delete_contact(&self, user_id: i64) -> Result<()>;
    async fn block_user(&self, peer_id: i64) -> Result<()>;
    async fn unblock_user(&self, peer_id: i64) -> Result<()>;
    async fn get_blocked_users(&self) -> Result<Vec<UserInfo>>;
    async fn import_contacts(&self, contacts: Vec<(String, String, String)>) -> Result<()>;
    async fn export_contacts(&self) -> Result<String>;
    async fn get_contact_ids(&self) -> Result<Vec<i64>>;
    async fn get_direct_chat_by_contact(&self, contact_query: &str) -> Result<i64>;
    async fn get_contact_chats(&self, contact_id: i64) -> Result<Vec<ChatInfo>>;
    async fn get_last_interaction(&self, contact_id: i64) -> Result<i32>;
}

#[async_trait]
pub trait FolderService {
    async fn list_folders(&self) -> Result<Vec<FolderInfo>>;
    async fn get_folder(&self, folder_id: i32) -> Result<FolderInfo>;
    async fn create_folder(&self, title: &str, included_peers: Vec<i64>, excluded_peers: Vec<i64>) -> Result<i32>;
    async fn delete_folder(&self, folder_id: i32) -> Result<()>;
    async fn add_chat_to_folder(&self, folder_id: i32, peer_id: i64) -> Result<()>;
    async fn remove_chat_from_folder(&self, folder_id: i32, peer_id: i64) -> Result<()>;
    async fn reorder_folders(&self, folder_ids: Vec<i32>) -> Result<()>;
}

#[async_trait]
pub trait SearchService {
    async fn search_global(&self, query: &str) -> Result<Vec<MessageInfo>>;
    async fn search_public_chats(&self, query: &str) -> Result<Vec<ChatInfo>>;
    async fn subscribe_public_channel(&self, query: &str) -> Result<()>;
    async fn get_common_chats(&self, user_id: i64) -> Result<Vec<ChatInfo>>;
    async fn list_topics(&self, peer_id: i64) -> Result<Vec<String>>;
    async fn get_privacy_settings(&self) -> Result<String>;
    async fn set_privacy_settings(&self, key: &str, value: &str) -> Result<()>;
}

#[async_trait]
pub trait AdminService {
    async fn get_participants(&self, peer_id: i64, limit: i32) -> Result<Vec<UserInfo>>;
    async fn ban_user(&self, peer_id: i64, user_id: i64) -> Result<()>;
    async fn promote_admin(&self, peer_id: i64, user_id: i64, rank: Option<&str>) -> Result<()>;
    async fn invite_users(&self, peer_id: i64, user_ids: Vec<i64>) -> Result<()>;
    async fn demote_admin(&self, peer_id: i64, user_id: i64) -> Result<()>;
    async fn edit_admin_rights(&self, peer_id: i64, user_id: i64, add_admins: bool, ban_users: bool, pin_messages: bool) -> Result<()>;
    async fn set_default_chat_permissions(&self, peer_id: i64, send_messages: bool, send_media: bool) -> Result<()>;
    async fn toggle_slow_mode(&self, peer_id: i64, seconds: i32) -> Result<()>;
    async fn get_admins(&self, peer_id: i64) -> Result<Vec<UserInfo>>;
    async fn get_banned_users(&self, peer_id: i64) -> Result<Vec<UserInfo>>;
    async fn get_recent_actions(&self, peer_id: i64, limit: i32) -> Result<Vec<String>>;
    async fn unban_user(&self, peer_id: i64, user_id: i64) -> Result<()>;
    async fn get_invite_link(&self, peer_id: i64) -> Result<String>;
    async fn export_chat_invite(&self, peer_id: i64) -> Result<String>;
    async fn import_chat_invite(&self, hash: &str) -> Result<i64>;
    async fn join_chat_by_link(&self, link: &str) -> Result<i64>;
}

// Redefine TelegramService to include AdminService as well
#[async_trait]
pub trait TelegramServiceFull: 
    AuthService + ChatService + MessageService + MediaService + ProfileService + BotService + ContactService + FolderService + AdminService
    + Send + Sync {}

pub mod grammers_impl;
pub mod path_guard;
