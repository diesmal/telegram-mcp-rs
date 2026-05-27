use crate::telegram::{ChatService, ChatInfo};
use super::GrammersService;
use anyhow::Result;
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl ChatService for GrammersService {
    async fn list_chats(&self, limit: i32) -> Result<Vec<ChatInfo>> {
        let mut chats = self.client.iter_dialogs();
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(dialog) = chats.next().await? {
            let peer = dialog.peer();
            result.push(ChatInfo {
                id: self.to_i64(peer.id()),
                title: peer.name().unwrap_or_default().to_string(),
                username: peer.username().map(|s| s.to_string()),
                chat_type: format!("{:?}", peer),
            });
            count += 1;
            if count >= limit {
                break;
            }
        }
        Ok(result)
    }

    async fn get_chat_info(&self, peer_id: i64) -> Result<ChatInfo> {
        let mut chats = self.client.iter_dialogs();
        while let Some(dialog) = chats.next().await? {
            let peer = dialog.peer();
            if self.to_i64(peer.id()) == peer_id {
                return Ok(ChatInfo {
                    id: self.to_i64(peer.id()),
                    title: peer.name().unwrap_or("").to_string(),
                    username: peer.username().map(|s| s.to_string()),
                    chat_type: format!("{:?}", peer),
                });
            }
        }
        anyhow::bail!("Chat not found in dialogs")
    }

    async fn create_group(&self, title: &str, user_ids: Vec<i64>) -> Result<i64> {
        let mut users = Vec::new();
        for id in user_ids {
            users.push(tl::enums::InputUser::User(tl::types::InputUser { user_id: id, access_hash: 0 }));
        }
        let _res = self.client.invoke(&tl::functions::messages::CreateChat {
            users,
            title: title.to_string(),
            ttl_period: None,
        }).await?;

        Ok(0)
    }

    async fn create_channel(&self, title: &str, about: &str, megagroup: bool) -> Result<i64> {
        let res = self.client.invoke(&tl::functions::channels::CreateChannel {
            broadcast: !megagroup,
            megagroup,
            for_import: false,
            title: title.to_string(),
            about: about.to_string(),
            geo_point: None,
            address: None,
            ttl_period: None,
            forum: false,
        }).await?;

        if let tl::enums::Updates::Updates(u) = res {
            for update in u.updates {
                if let tl::enums::Update::Channel(c) = update {
                    return Ok(-c.channel_id - 1001000000000); // Channel format
                }
            }
        }
        anyhow::bail!("Failed to extract channel ID from response")
    }

    async fn leave_chat(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Chat(c) => {
                self.client.invoke(&tl::functions::messages::DeleteChatUser {
                    chat_id: c.chat_id,
                    user_id: tl::enums::InputUser::UserSelf,
                    revoke_history: false,
                }).await?;
            }
            tl::enums::InputPeer::Channel(c) => {
                self.client.invoke(&tl::functions::channels::LeaveChannel {
                    channel: tl::enums::InputChannel::Channel(tl::types::InputChannel {
                        channel_id: c.channel_id,
                        access_hash: c.access_hash,
                    }),
                }).await?;
            }
            _ => anyhow::bail!("Can only leave groups or channels"),
        }
        Ok(())
    }

    async fn set_chat_muted(&self, peer_id: i64, muted: bool, until: Option<i32>) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let settings = tl::types::InputPeerNotifySettings {
            show_previews: None,
            silent: None,
            mute_until: Some(if muted { until.unwrap_or(2147483647) } else { 0 }),
            sound: None,
            stories_muted: None,
            stories_hide_sender: None,
            stories_sound: None,
        };

        self.client.invoke(&tl::functions::account::UpdateNotifySettings {
            peer: tl::enums::InputNotifyPeer::Peer(tl::types::InputNotifyPeer { peer: peer_ref }),
            settings: tl::enums::InputPeerNotifySettings::Settings(settings),
        }).await?;
        Ok(())
    }

    async fn set_chat_archived(&self, peer_id: i64, archived: bool) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let folder_id = if archived { 1 } else { 0 };
        self.client.invoke(&tl::functions::folders::EditPeerFolders {
            folder_peers: vec![tl::enums::InputFolderPeer::Peer(tl::types::InputFolderPeer {
                peer: peer_ref,
                folder_id,
            })],
        }).await?;
        Ok(())
    }

    async fn edit_chat_about(&self, peer_id: i64, about: &str) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Channel(c) => {
                self.client.invoke(&tl::functions::messages::EditChatAbout {
                    peer: tl::enums::InputPeer::Channel(c),
                    about: about.to_string(),
                }).await?;
            }
            tl::enums::InputPeer::Chat(c) => {
                self.client.invoke(&tl::functions::messages::EditChatAbout {
                    peer: tl::enums::InputPeer::Chat(c),
                    about: about.to_string(),
                }).await?;
            }
            _ => anyhow::bail!("Invalid peer for editing about"),
        }
        Ok(())
    }

    async fn edit_chat_title(&self, peer_id: i64, title: &str) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Channel(c) => {
                self.client.invoke(&tl::functions::channels::EditTitle {
                    channel: tl::enums::InputChannel::Channel(tl::types::InputChannel { channel_id: c.channel_id, access_hash: c.access_hash }),
                    title: title.to_string(),
                }).await?;
            }
            tl::enums::InputPeer::Chat(c) => {
                self.client.invoke(&tl::functions::messages::EditChatTitle {
                    chat_id: c.chat_id,
                    title: title.to_string(),
                }).await?;
            }
            _ => anyhow::bail!("Invalid peer for editing title"),
        }
        Ok(())
    }

    async fn edit_chat_photo(&self, _peer_id: i64, _path: &str) -> Result<()> {
        anyhow::bail!("Edit chat photo requires MTProto v2 mapping")
    }

    async fn delete_chat_photo(&self, _peer_id: i64) -> Result<()> {
        anyhow::bail!("Delete chat photo requires MTProto v2 mapping")
    }
}
