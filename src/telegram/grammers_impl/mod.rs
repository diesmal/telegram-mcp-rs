use grammers_client::Client;
use crate::telegram::path_guard::PathGuard;
use grammers_session::types::PeerId;
use grammers_session::Session;
use anyhow::Result;
use grammers_tl_types as tl;
use std::sync::Arc;

pub mod auth;
pub mod chats;
pub mod convert;
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
    pub session: Arc<dyn Session>,
    pub path_guard: PathGuard,
}

impl GrammersService {
    pub fn new(client: Client, session: Arc<dyn Session>, path_guard: PathGuard) -> Self {
        Self { client, session, path_guard }
    }

    /// Builds an InputPeer from a Bot-API style id using the access hash cached
    /// in the session (hash 0 if the peer was never seen).
    async fn get_peer_ref(&self, id: i64) -> Result<tl::enums::InputPeer> {
        // PeerId constructors panic on out-of-range ids, so validate first.
        let peer_id = if (1..=0xffffffffff).contains(&id) {
            PeerId::user(id)
        } else if id <= -1_000_000_000_000 {
            let bare = -id - 1_000_000_000_000;
            if !((1..=997852516352).contains(&bare) || (1002147483649..=3000000000000).contains(&bare)) {
                anyhow::bail!("Invalid channel id {id}");
            }
            PeerId::channel(bare)
        } else if (-999999999999..=-1).contains(&id) {
            PeerId::chat(-id)
        } else {
            anyhow::bail!("Invalid peer id {id}. Use ids from list_chats / resolve_username.");
        };
        let access_hash = match self.session.peer(peer_id).await {
            Some(info) => info.auth().map(|a| a.hash()).unwrap_or(0),
            None => {
                tracing::warn!("get_peer_ref({}): peer not in session cache", id);
                0
            }
        };

        if id > 0 {
            Ok(tl::enums::InputPeer::User(tl::types::InputPeerUser { user_id: id, access_hash }))
        } else if id <= -1_000_000_000_000 {
            Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { channel_id: -id - 1_000_000_000_000, access_hash }))
        } else {
            Ok(tl::enums::InputPeer::Chat(tl::types::InputPeerChat { chat_id: -id }))
        }
    }

    fn to_i64(&self, id: PeerId) -> i64 {
        id.bot_api_dialog_id()
    }
}

impl crate::telegram::TelegramService for GrammersService {}
