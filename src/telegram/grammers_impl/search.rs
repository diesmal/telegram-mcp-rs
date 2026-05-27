use crate::telegram::{SearchService, MessageInfo, ChatInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl SearchService for GrammersService {
    async fn search_global(&self, _query: &str) -> Result<Vec<MessageInfo>> {
        anyhow::bail!("Global search requires MTProto v2 mapping")
    }

    async fn search_public_chats(&self, query: &str) -> Result<Vec<ChatInfo>> {
        let res = self.client.invoke(&tl::functions::contacts::Search {
            q: query.to_string(),
            limit: 50,
        }).await?;
        
        let mut result = Vec::new();
        if let tl::enums::contacts::Found::Found(f) = res {
            for chat in f.chats {
                match chat {
                    tl::enums::Chat::Channel(c) => {
                        result.push(ChatInfo {
                            id: c.id,
                            title: c.title,
                            username: c.username,
                            chat_type: "Channel".to_string(),
                        });
                    }
                    tl::enums::Chat::Chat(c) => {
                        result.push(ChatInfo {
                            id: c.id,
                            title: c.title,
                            username: None,
                            chat_type: "Group".to_string(),
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
    }

    async fn subscribe_public_channel(&self, query: &str) -> Result<()> {
        let peer = self.client.resolve_username(query).await?.context("Channel not found")?;
        let peer_ref = self.get_peer_ref(peer.id().bot_api_dialog_id()).await?;
        if let tl::enums::InputPeer::Channel(c) = peer_ref {
            self.client.invoke(&tl::functions::channels::JoinChannel {
                channel: tl::enums::InputChannel::Channel(tl::types::InputChannel {
                    channel_id: c.channel_id,
                    access_hash: c.access_hash,
                })
            }).await?;
        }
        Ok(())
    }

    async fn get_common_chats(&self, user_id: i64) -> Result<Vec<ChatInfo>> {
        let user = tl::enums::InputUser::User(tl::types::InputUser { user_id, access_hash: 0 });
        let res = self.client.invoke(&tl::functions::messages::GetCommonChats {
            user_id: user,
            max_id: 0,
            limit: 50,
        }).await?;
        
        let mut result = Vec::new();
        if let tl::enums::messages::Chats::Chats(chats) = res {
            for chat in chats.chats {
                match chat {
                    tl::enums::Chat::Chat(c) => {
                        result.push(ChatInfo { id: c.id, title: c.title, username: None, chat_type: "Group".to_string() });
                    }
                    _ => {}
                }
            }
        }
        Ok(result)
    }

    async fn list_topics(&self, _peer_id: i64) -> Result<Vec<String>> {
        anyhow::bail!("Topics are only available in supergroups/channels")
    }

    async fn get_privacy_settings(&self) -> Result<String> {
        Ok("Privacy settings not implemented via TL in this stub, use official clients".to_string())
    }

    async fn set_privacy_settings(&self, _key: &str, _value: &str) -> Result<()> {
        Ok(())
    }
}
