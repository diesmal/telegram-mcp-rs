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
        let mut dialogs = self.client.iter_dialogs();
        while let Some(dialog) = dialogs.next().await? {
            let peer = dialog.peer();
            if self.to_i64(peer.id()) == id {
                match peer {
                    grammers_client::peer::Peer::User(u) => {
                        let access_hash = match &u.raw {
                            tl::enums::User::User(user) => user.access_hash.unwrap_or(0),
                            _ => 0,
                        };
                        return Ok(tl::enums::InputPeer::User(tl::types::InputPeerUser { 
                            user_id: u.raw.id(), 
                            access_hash 
                        }));
                    }
                    grammers_client::peer::Peer::Group(g) => {
                        match &g.raw {
                            tl::enums::Chat::Chat(chat) => {
                                return Ok(tl::enums::InputPeer::Chat(tl::types::InputPeerChat { chat_id: chat.id }));
                            }
                            tl::enums::Chat::Channel(channel) => {
                                return Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { 
                                    channel_id: channel.id, 
                                    access_hash: channel.access_hash.unwrap_or(0) 
                                }));
                            }
                            tl::enums::Chat::ChannelForbidden(channel) => {
                                return Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { 
                                    channel_id: channel.id, 
                                    access_hash: channel.access_hash 
                                }));
                            }
                            _ => {}
                        }
                    }
                    grammers_client::peer::Peer::Channel(c) => {
                        return Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { 
                            channel_id: c.raw.id, 
                            access_hash: c.raw.access_hash.unwrap_or(0) 
                        }));
                    }
                }
            }
        }

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
