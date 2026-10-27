//! Conversions from grammers / raw TL objects into the lean `*Info` models.
//!
//! Everything returned to the LLM goes through here, so keep it small: short
//! labels instead of Debug dumps, names instead of bare ids, no access hashes.

use crate::telegram::{ChatInfo, MessageInfo};
use grammers_client::message::Message;
use grammers_client::peer::Peer;
use grammers_tl_types as tl;
use std::collections::HashMap;

/// Short kind label plus member count (when Telegram includes it).
pub fn peer_kind(peer: &Peer) -> (&'static str, Option<i32>) {
    match peer {
        Peer::User(u) => (if u.is_bot() { "bot" } else { "user" }, None),
        Peer::Group(g) => match &g.raw {
            tl::enums::Chat::Chat(c) => ("group", Some(c.participants_count)),
            tl::enums::Chat::Channel(c) => ("supergroup", c.participants_count),
            _ => ("group", None),
        },
        Peer::Channel(c) => ("channel", c.raw.participants_count),
    }
}

pub fn chat_from_peer(peer: &Peer, unread: Option<i32>) -> ChatInfo {
    let (kind, members) = peer_kind(peer);
    ChatInfo {
        id: peer.id().bot_api_dialog_id(),
        title: peer.name().unwrap_or_default().to_string(),
        username: peer.username().map(str::to_string),
        kind: kind.to_string(),
        members,
        unread,
    }
}

pub fn chat_from_raw(chat: &tl::enums::Chat) -> Option<ChatInfo> {
    match chat {
        tl::enums::Chat::Chat(c) => Some(ChatInfo {
            id: -c.id,
            title: c.title.clone(),
            kind: "group".into(),
            members: Some(c.participants_count),
            ..Default::default()
        }),
        tl::enums::Chat::Channel(c) => Some(ChatInfo {
            id: -1_000_000_000_000 - c.id,
            title: c.title.clone(),
            username: c.username.clone(),
            kind: if c.megagroup { "supergroup" } else { "channel" }.into(),
            members: c.participants_count,
            ..Default::default()
        }),
        _ => None,
    }
}

/// Short human label for message media, `None` for no media / link previews.
pub fn media_label(media: &tl::enums::MessageMedia) -> Option<String> {
    use tl::enums::MessageMedia as M;
    Some(match media {
        M::Empty | M::WebPage(_) | M::Unsupported => return None,
        M::Photo(_) => "photo".into(),
        M::Geo(_) | M::GeoLive(_) | M::Venue(_) => "location".into(),
        M::Contact(c) => format!("contact: {} {}", c.first_name, c.last_name).trim_end().to_string(),
        M::Poll(p) => match &p.poll {
            tl::enums::Poll::Poll(poll) => match &poll.question {
                tl::enums::TextWithEntities::Entities(t) => format!("poll: {}", t.text),
            },
        },
        M::Document(d) => match &d.document {
            Some(tl::enums::Document::Document(doc)) => document_label(doc),
            _ => "file".into(),
        },
        M::Dice(d) => format!("dice {}", d.emoticon),
        M::Game(_) => "game".into(),
        M::Invoice(_) => "invoice".into(),
        M::Story(_) => "story".into(),
        M::Giveaway(_) | M::GiveawayResults(_) => "giveaway".into(),
        M::PaidMedia(_) => "paid media".into(),
        #[allow(unreachable_patterns)]
        _ => "media".into(),
    })
}

fn document_label(doc: &tl::types::Document) -> String {
    use tl::enums::DocumentAttribute as A;
    let mut file_name = None;
    let mut kind = None;
    for attr in &doc.attributes {
        match attr {
            A::Filename(f) => file_name = Some(f.file_name.clone()),
            A::Sticker(s) => kind = Some(format!("sticker {}", s.alt)),
            A::Animated => kind = kind.or(Some("gif".into())),
            A::Video(v) => {
                if kind.is_none() {
                    kind = Some(if v.round_message { format!("video note {}s", v.duration as i64) } else { format!("video {}s", v.duration as i64) });
                }
            }
            A::Audio(a) => {
                kind = Some(if a.voice { format!("voice {}s", a.duration) } else { format!("audio {}s", a.duration) });
            }
            _ => {}
        }
    }
    match (kind, file_name) {
        (Some(k), _) => k,
        (None, Some(f)) => format!("file: {f}"),
        (None, None) => format!("file ({})", doc.mime_type),
    }
}

/// From a high-level grammers message (history, context, sends).
pub fn msg_from_client(msg: &Message) -> MessageInfo {
    let media = match &msg.raw {
        tl::enums::Message::Message(m) => m.media.as_ref().and_then(media_label),
        _ => None,
    };
    let text = match &msg.raw {
        tl::enums::Message::Service(_) => "[service message]".to_string(),
        _ => msg.text().to_string(),
    };
    // Channel posts have no sender (the channel is the author): leave it empty
    // rather than repeating the channel name on every line.
    let (sender_id, sender_name) = if msg.outgoing() {
        (0, Some("me".to_string()))
    } else {
        match msg.sender_id() {
            Some(id) => (id.bot_api_dialog_id(), msg.sender().and_then(|s| s.name()).map(str::to_string)),
            None => (0, None),
        }
    };
    MessageInfo {
        id: msg.id(),
        sender_id,
        sender_name,
        text,
        date: msg.date().timestamp(),
        reply_to_msg_id: msg.reply_to_message_id(),
        media,
        ..Default::default()
    }
}

/// Lookup tables for the `users` / `chats` vectors that raw TL results carry.
#[derive(Default)]
pub struct Names {
    users: HashMap<i64, String>,
    chats: HashMap<i64, String>,
}

impl Names {
    pub fn new(users: &[tl::enums::User], chats: &[tl::enums::Chat]) -> Self {
        let mut n = Names::default();
        for u in users {
            if let tl::enums::User::User(u) = u {
                let name = match (&u.first_name, &u.last_name) {
                    (Some(f), Some(l)) => format!("{f} {l}"),
                    (Some(f), None) => f.clone(),
                    (None, Some(l)) => l.clone(),
                    (None, None) => u.username.clone().unwrap_or_else(|| "Deleted Account".into()),
                };
                n.users.insert(u.id, name);
            }
        }
        for c in chats {
            if let Some(info) = chat_from_raw(c) {
                n.chats.insert(info.id, info.title);
            }
        }
        n
    }

    pub fn peer_label(&self, peer: &tl::enums::Peer) -> (i64, Option<String>) {
        self.peer(peer)
    }

    fn peer(&self, peer: &tl::enums::Peer) -> (i64, Option<String>) {
        match peer {
            tl::enums::Peer::User(u) => (u.user_id, self.users.get(&u.user_id).cloned()),
            tl::enums::Peer::Chat(c) => (-c.chat_id, self.chats.get(&-c.chat_id).cloned()),
            tl::enums::Peer::Channel(c) => {
                let id = -1_000_000_000_000 - c.channel_id;
                (id, self.chats.get(&id).cloned())
            }
        }
    }
}

/// From raw `messages.Messages` results (search, pinned, scheduled).
/// `with_chat` adds the chat id/title to each message (cross-chat results).
pub fn msgs_from_raw(res: tl::enums::messages::Messages, with_chat: bool) -> Vec<MessageInfo> {
    let (messages, users, chats) = match res {
        tl::enums::messages::Messages::Messages(m) => (m.messages, m.users, m.chats),
        tl::enums::messages::Messages::Slice(s) => (s.messages, s.users, s.chats),
        tl::enums::messages::Messages::ChannelMessages(c) => (c.messages, c.users, c.chats),
        tl::enums::messages::Messages::NotModified(_) => return Vec::new(),
    };
    let names = Names::new(&users, &chats);
    messages
        .into_iter()
        .filter_map(|m| match m {
            tl::enums::Message::Message(mm) => Some(mm),
            _ => None,
        })
        .map(|mm| {
            // Incoming private messages carry no from_id (the sender is the chat
            // itself); channel posts carry none either and get no sender.
            let (sender_id, sender_name) = if mm.out {
                (0, Some("me".to_string()))
            } else {
                match (&mm.from_id, &mm.peer_id) {
                    (Some(from), _) => names.peer(from),
                    (None, peer @ tl::enums::Peer::User(_)) => names.peer(peer),
                    (None, _) => (0, None),
                }
            };
            let (chat_id, chat_title) = if with_chat {
                let (id, title) = names.peer(&mm.peer_id);
                (Some(id), title)
            } else {
                (None, None)
            };
            let reply_to_msg_id = match &mm.reply_to {
                Some(tl::enums::MessageReplyHeader::Header(h)) => h.reply_to_msg_id,
                _ => None,
            };
            MessageInfo {
                id: mm.id,
                sender_id,
                sender_name,
                media: mm.media.as_ref().and_then(media_label),
                text: mm.message,
                date: mm.date as i64,
                reply_to_msg_id,
                chat_id,
                chat_title,
            }
        })
        .collect()
}
