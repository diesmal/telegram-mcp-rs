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
        let access_hash = {
            let tmp_path = format!("telegram_{}.session.tmp", id);
            let _ = std::fs::copy("telegram.session", &tmp_path);
            let result = match libsql::Builder::new_local(&tmp_path).build().await {
                Ok(db) => {
                    match db.connect() {
                        Ok(conn) => {
                            match conn.query("SELECT hash FROM peer_info WHERE peer_id = ?1", vec![libsql::Value::Integer(id)]).await {
                                Ok(mut rows) => {
                                    if let Ok(Some(row)) = rows.next().await {
                                        let h = row.get::<i64>(0).unwrap_or(0);
                                        tracing::info!("get_peer_ref({}): libsql returned {}", id, h);
                                        h
                                    } else {
                                        tracing::error!("get_peer_ref({}): no rows returned", id);
                                        0
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("get_peer_ref({}): query error: {:?}", id, e);
                                    0
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("get_peer_ref({}): connect error: {:?}", id, e);
                            0
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("get_peer_ref({}): builder error: {:?}", id, e);
                    0
                }
            };
            let _ = std::fs::remove_file(&tmp_path);
            result
        };

        if id > 0 {
            Ok(tl::enums::InputPeer::User(tl::types::InputPeerUser { user_id: id, access_hash }))
        } else if id <= -1000000000000 {
            Ok(tl::enums::InputPeer::Channel(tl::types::InputPeerChannel { channel_id: -id - 1000000000000, access_hash }))
        } else {
            Ok(tl::enums::InputPeer::Chat(tl::types::InputPeerChat { chat_id: -id }))
        }
    }

    fn to_i64(&self, id: PeerId) -> i64 {
        id.bot_api_dialog_id()
    }
}

impl crate::telegram::TelegramService for GrammersService {}
