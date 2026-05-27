use grammers_client::Client;
use crate::telegram::path_guard::PathGuard;
use grammers_session::types::PeerId;
use anyhow::{Result, Context};
use grammers_tl_types as tl;

pub mod auth;
pub mod chats;
pub mod messages;
pub mod media;
pub mod profile;
pub mod bots;
pub mod contacts;
pub mod folders;
pub mod admin;
pub mod search;

pub struct GrammersService {
    pub client: Client,
    pub path_guard: PathGuard,
}

impl GrammersService {
    pub fn new(client: Client, path_guard: PathGuard) -> Self {
        Self { client, path_guard }
    }

    async fn get_peer_ref(&self, id: i64) -> Result<tl::enums::InputPeer> {
        if id > 0 {
            Ok(tl::enums::InputPeer::User(tl::types::InputPeerUser { user_id: id, access_hash: 0 }))
        } else if id < -1001000000000 {
            Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { channel_id: -id - 1001000000000, access_hash: 0 }))
        } else {
            Ok(tl::enums::InputPeer::Chat(tl::types::InputPeerChat { chat_id: -id }))
        }
    }

    fn to_i64(&self, id: PeerId) -> i64 {
        id.bot_api_dialog_id()
    }
}

impl crate::telegram::TelegramService for GrammersService {}
