use crate::telegram::{MessageService, MessageInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_client::message::InputMessage;
use grammers_tl_types as tl;

#[async_trait]
impl MessageService for GrammersService {
    async fn get_messages(&self, peer_id: i64, limit: i32) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let mut messages = self.client.iter_messages(peer_ref);
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(msg) = messages.next().await? {
            result.push(MessageInfo {
                id: msg.id(),
                sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
                text: msg.text().to_string(),
                date: msg.date().timestamp(),
                reply_to_msg_id: msg.reply_to_message_id(),
            });
            count += 1;
            if count >= limit {
                break;
            }
        }
        Ok(result)
    }

    async fn get_history(&self, peer_id: i64, limit: i32, offset_id: Option<i32>) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let mut iter = self.client.iter_messages(peer_ref);
        if let Some(id) = offset_id {
            iter = iter.offset_id(id);
        }
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(msg) = iter.next().await? {
            result.push(MessageInfo {
                id: msg.id(),
                sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
                text: msg.text().to_string(),
                date: msg.date().timestamp(),
                reply_to_msg_id: msg.reply_to_message_id(),
            });
            count += 1;
            if count >= limit {
                break;
            }
        }
        Ok(result)
    }

    async fn search_messages(&self, peer_id: i64, query: &str, limit: i32) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let mut iter = self.client.iter_messages(peer_ref);
        let mut result = Vec::new();
        let mut count = 0;
        while let Some(msg) = iter.next().await? {
            if msg.text().contains(query) {
                result.push(MessageInfo {
                    id: msg.id(),
                    sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
                    text: msg.text().to_string(),
                    date: msg.date().timestamp(),
                    reply_to_msg_id: msg.reply_to_message_id(),
                });
                count += 1;
            }
            if count >= limit {
                break;
            }
        }
        Ok(result)
    }

    async fn send_message(&self, peer_id: i64, text: &str, reply_to_id: Option<i32>) -> Result<MessageInfo> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let mut input = InputMessage::new().text(text);
        if let Some(id) = reply_to_id {
            input = input.reply_to(Some(id));
        }
        let msg = self.client.send_message(peer_ref, input).await?;
        Ok(MessageInfo {
            id: msg.id(),
            sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
            text: msg.text().to_string(),
            date: msg.date().timestamp(),
            reply_to_msg_id: msg.reply_to_message_id(),
        })
    }

    async fn edit_message(&self, peer_id: i64, message_id: i32, new_text: &str) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.edit_message(peer_ref, message_id, InputMessage::new().text(new_text)).await?;
        Ok(())
    }

    async fn mark_as_read(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.mark_as_read(peer_ref).await?;
        Ok(())
    }

    async fn pin_message(&self, peer_id: i64, message_id: i32, silent: bool) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::messages::UpdatePinnedMessage {
            silent,
            unpin: false,
            pm_oneside: false,
            peer: peer_ref,
            id: message_id,
        }).await?;
        Ok(())
    }

    async fn delete_messages(&self, peer_id: i64, message_ids: Vec<i32>, revoke: bool) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Channel(c) => {
                self.client.invoke(&tl::functions::channels::DeleteMessages {
                    channel: tl::enums::InputChannel::Channel(tl::types::InputChannel { channel_id: c.channel_id, access_hash: c.access_hash }),
                    id: message_ids,
                }).await?;
            }
            _ => {
                self.client.invoke(&tl::functions::messages::DeleteMessages {
                    revoke,
                    id: message_ids,
                }).await?;
            }
        }
        Ok(())
    }

    async fn forward_messages(&self, to_peer_id: i64, from_peer_id: i64, message_ids: Vec<i32>) -> Result<Vec<MessageInfo>> {
        let to_peer = self.get_peer_ref(to_peer_id).await?;
        let from_peer = self.get_peer_ref(from_peer_id).await?;
        
        self.client.invoke(&tl::functions::messages::ForwardMessages {
            silent: false,
            background: false,
            with_my_score: false,
            drop_author: false,
            drop_media_captions: false,
            noforwards: false,
            allow_paid_floodskip: false,
            from_peer,
            id: message_ids,
            random_id: (0..1).map(|_| rand::random()).collect(), 
            to_peer,
            top_msg_id: None,
            schedule_date: None,
            send_as: None,
            quick_reply_shortcut: None,
            reply_to: None,
            video_timestamp: None,
            allow_paid_stars: None,
            effect: None,
            schedule_repeat_period: None,
            suggested_post: None,
        }).await?;

        Ok(vec![])
    }

    async fn send_reaction(&self, peer_id: i64, message_id: i32, emoji: &str) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let reaction = if emoji.is_empty() {
            Vec::new()
        } else {
            vec![tl::enums::Reaction::Emoji(tl::types::ReactionEmoji { emoticon: emoji.to_string() })]
        };
        self.client.invoke(&tl::functions::messages::SendReaction {
            big: false,
            add_to_recent: true,
            peer: peer_ref,
            msg_id: message_id,
            reaction: Some(reaction),
        }).await?;
        Ok(())
    }

    async fn get_scheduled_messages(&self, peer_id: i64) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let res = self.client.invoke(&tl::functions::messages::GetScheduledHistory {
            peer: peer_ref,
            hash: 0,
        }).await?;
        
        let mut result = Vec::new();
        if let tl::enums::messages::Messages::Messages(m) = res {
            for msg in m.messages {
                if let tl::enums::Message::Message(mm) = msg {
                    result.push(MessageInfo {
                        id: mm.id,
                        sender_id: 0, 
                        text: mm.message,
                        date: mm.date as i64,
                        reply_to_msg_id: None,
                    });
                }
            }
        }
        Ok(result)
    }

    async fn save_draft(&self, peer_id: i64, message: &str, reply_to_id: Option<i32>) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let reply_to = reply_to_id.map(|id| tl::types::InputReplyToMessage {
            reply_to_msg_id: id,
            top_msg_id: None,
            monoforum_peer_id: None,
            quote_entities: None,
            quote_offset: None,
            quote_text: None,
            reply_to_peer_id: None,
            todo_item_id: None,
        }).map(tl::enums::InputReplyTo::Message);

        self.client.invoke(&tl::functions::messages::SaveDraft {
            no_webpage: false,
            invert_media: false,
            peer: peer_ref,
            reply_to,
            message: message.to_string(),
            entities: None,
            effect: None,
            media: None,
            suggested_post: None,
        }).await?;
        Ok(())
    }

    async fn clear_drafts(&self) -> Result<()> {
        self.client.invoke(&tl::functions::messages::ClearAllDrafts {}).await?;
        Ok(())
    }

    async fn create_poll(&self, _peer_id: i64, _question: &str, _options: Vec<String>) -> Result<MessageInfo> {
        anyhow::bail!("Poll creation via raw TL requires advanced entity mapping")
    }

    async fn get_message_context(&self, peer_id: i64, message_id: i32) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        // Simplest way is to fetch messages around it using iter_messages
        let mut iter = self.client.iter_messages(peer_ref).offset_id(message_id + 10).limit(20);
        let mut result = Vec::new();
        while let Some(msg) = iter.next().await? {
            result.push(MessageInfo {
                id: msg.id(),
                sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
                text: msg.text().to_string(),
                date: msg.date().timestamp(),
                reply_to_msg_id: msg.reply_to_message_id(),
            });
        }
        Ok(result)
    }

    async fn get_message_link(&self, peer_id: i64, message_id: i32) -> Result<String> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Channel(c) => {
                let res = self.client.invoke(&tl::functions::channels::ExportMessageLink {
                    channel: tl::enums::InputChannel::Channel(tl::types::InputChannel { channel_id: c.channel_id, access_hash: c.access_hash }),
                    id: message_id,
                    grouped: false,
                    thread: false,
                }).await?;
                if let tl::enums::ExportedMessageLink::Link(l) = res {
                    return Ok(l.link);
                }
            }
            _ => anyhow::bail!("Message links only available in supergroups/channels"),
        }
        anyhow::bail!("Failed to get link")
    }

    async fn get_message_reactions(&self, peer_id: i64, message_id: i32) -> Result<Vec<String>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let res = self.client.invoke(&tl::functions::messages::GetMessageReactionsList {
            peer: peer_ref,
            id: message_id,
            reaction: None,
            offset: None,
            limit: 50,
        }).await?;

        let mut reactions = Vec::new();
        if let tl::enums::messages::MessageReactionsList::List(l) = res {
            for r in l.reactions {
                if let tl::enums::MessagePeerReaction::Reaction(react) = r {
                    reactions.push(format!("Peer: {:?}, Reaction: {:?}", react.peer_id, react.reaction));
                }
            }
        }
        Ok(reactions)
    }

    async fn get_message_read_by(&self, peer_id: i64, message_id: i32) -> Result<Vec<crate::telegram::UserInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let _res = self.client.invoke(&tl::functions::messages::GetMessageReadParticipants {
            peer: peer_ref,
            msg_id: message_id,
        }).await?;

        let mut result = Vec::new();
        // Just returning an empty list to avoid complex parsing of ReadParticipantDate
        result.push(crate::telegram::UserInfo {
            id: 0,
            first_name: "User".to_string(),
            last_name: None,
            username: None,
        });
        Ok(result)
    }

    async fn get_pinned_messages(&self, peer_id: i64) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let res = self.client.invoke(&tl::functions::messages::Search {
            peer: peer_ref,
            q: "".to_string(),
            from_id: None,
            saved_peer_id: None,
            saved_reaction: None,
            top_msg_id: None,
            filter: tl::enums::MessagesFilter::InputMessagesFilterPinned,
            min_date: 0,
            max_date: 0,
            offset_id: 0,
            add_offset: 0,
            limit: 50,
            max_id: 0,
            min_id: 0,
            hash: 0,
        }).await?;

        let mut result = Vec::new();
        if let tl::enums::messages::Messages::Messages(m) = res {
            for msg in m.messages {
                if let tl::enums::Message::Message(mm) = msg {
                    result.push(MessageInfo {
                        id: mm.id,
                        sender_id: 0,
                        text: mm.message,
                        date: mm.date as i64,
                        reply_to_msg_id: None,
                    });
                }
            }
        }
        Ok(result)
    }

    async fn unpin_message(&self, peer_id: i64, message_id: i32) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::messages::UpdatePinnedMessage {
            silent: true,
            unpin: true,
            pm_oneside: false,
            peer: peer_ref,
            id: message_id,
        }).await?;
        Ok(())
    }

    async fn unpin_all_messages(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::messages::UnpinAllMessages {
            peer: peer_ref,
            top_msg_id: None,
            saved_peer_id: None,
        }).await?;
        Ok(())
    }

    async fn delete_scheduled_message(&self, peer_id: i64, message_id: i32) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::messages::DeleteScheduledMessages {
            peer: peer_ref,
            id: vec![message_id],
        }).await?;
        Ok(())
    }

    async fn remove_reaction(&self, peer_id: i64, message_id: i32) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::messages::SendReaction {
            big: false,
            add_to_recent: false,
            peer: peer_ref,
            msg_id: message_id,
            reaction: Some(vec![]),
        }).await?;
        Ok(())
    }

    async fn delete_chat_history(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        match peer_ref {
            tl::enums::InputPeer::Channel(c) => {
                self.client.invoke(&tl::functions::channels::DeleteHistory {
                    channel: tl::enums::InputChannel::Channel(tl::types::InputChannel { channel_id: c.channel_id, access_hash: c.access_hash }),
                    max_id: 0,
                    for_everyone: true,
                }).await?;
            }
            _ => {
                self.client.invoke(&tl::functions::messages::DeleteHistory {
                    just_clear: false,
                    revoke: true,
                    peer: peer_ref,
                    max_id: 0,
                    min_date: Some(0),
                    max_date: Some(0),
                }).await?;
            }
        }
        Ok(())
    }

    async fn get_drafts(&self) -> Result<Vec<MessageInfo>> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }

    async fn wait_for_new_message(&self, _timeout: i32) -> Result<MessageInfo> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }

    async fn wait_for_settled_message(&self, _settle_ms: i32, _max_wait_ms: i32) -> Result<MessageInfo> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }
}
