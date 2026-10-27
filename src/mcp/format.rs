//! Compact, LLM-oriented rendering of tool results.
//!
//! Lists are rendered one item per line instead of pretty JSON: no repeated
//! keys, no indentation, no null fields. Long message texts are truncated
//! unless the caller asks for `full` output.

use crate::telegram::{BotInfo, ChatInfo, ContactInfo, FolderInfo, MessageInfo, UserInfo};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// Message text longer than this (in chars) is cut unless `full` is set.
pub const TRUNCATE_AT: usize = 300;

pub trait Render {
    fn render(&self, full: bool) -> String;

    /// How a list of these is rendered; default is one `render` per line.
    fn render_list(items: &[Self], full: bool) -> String
    where
        Self: Sized,
    {
        items.iter().map(|i| i.render(full)).collect::<Vec<_>>().join("\n")
    }
}

impl<T: Render> Render for Vec<T> {
    fn render(&self, full: bool) -> String {
        if self.is_empty() {
            return "(none)".to_string();
        }
        T::render_list(self, full)
    }
}

impl Render for () {
    fn render(&self, _full: bool) -> String {
        "OK".to_string()
    }
}

impl Render for String {
    fn render(&self, _full: bool) -> String {
        self.clone()
    }
}

impl Render for i64 {
    fn render(&self, _full: bool) -> String {
        self.to_string()
    }
}

impl Render for i32 {
    fn render(&self, _full: bool) -> String {
        self.to_string()
    }
}

impl Render for PathBuf {
    fn render(&self, _full: bool) -> String {
        self.display().to_string()
    }
}

fn opt_username(u: &Option<String>) -> String {
    u.as_deref().map(|u| format!(" @{u}")).unwrap_or_default()
}

impl Render for ChatInfo {
    fn render(&self, _full: bool) -> String {
        let mut s = format!("{} | {} | {}{}", self.id, self.kind, self.title, opt_username(&self.username));
        if let Some(m) = self.members {
            s.push_str(&format!(" | {m} members"));
        }
        if let Some(u) = self.unread {
            s.push_str(&format!(" | {u} unread"));
        }
        s
    }
}

fn full_name(first: &str, last: &Option<String>) -> String {
    match last.as_deref() {
        Some(l) if !l.is_empty() => format!("{first} {l}"),
        _ => first.to_string(),
    }
}

impl Render for UserInfo {
    fn render(&self, _full: bool) -> String {
        format!("{} | {}{}", self.id, full_name(&self.first_name, &self.last_name), opt_username(&self.username))
    }
}

impl Render for ContactInfo {
    fn render(&self, full: bool) -> String {
        let mut s = format!("{} | {}{}", self.user_id, full_name(&self.first_name, &self.last_name), opt_username(&self.username));
        if full {
            if let Some(p) = self.phone.as_deref().filter(|p| !p.is_empty()) {
                s.push_str(&format!(" | +{p}"));
            }
        }
        s
    }
}

impl Render for FolderInfo {
    fn render(&self, full: bool) -> String {
        let mut s = format!("{} | {}{}", self.id, self.emoticon.as_deref().map(|e| format!("{e} ")).unwrap_or_default(), self.title);
        if full {
            s.push_str(&format!(" | pinned {:?} | included {:?} | excluded {:?}", self.pinned_peers, self.included_peers, self.excluded_peers));
        } else {
            s.push_str(&format!(" | {} chats", self.pinned_peers.len() + self.included_peers.len()));
        }
        s
    }
}

impl Render for BotInfo {
    fn render(&self, _full: bool) -> String {
        let mut parts = vec![self.name.clone()];
        if !self.about.is_empty() {
            parts.push(format!("About: {}", self.about));
        }
        if !self.description.is_empty() && self.description != self.about {
            parts.push(format!("Description: {}", self.description));
        }
        parts.join("\n")
    }
}

pub fn format_date(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|d| d.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn message_text(text: &str, full: bool) -> String {
    let flat = text.replace("\r\n", "\n").replace('\n', " ⏎ ");
    if full {
        return flat;
    }
    let total = flat.chars().count();
    if total <= TRUNCATE_AT {
        return flat;
    }
    let cut: String = flat.chars().take(TRUNCATE_AT).collect();
    format!("{cut}…[+{} chars]", total - TRUNCATE_AT)
}

/// `None` when there is nothing useful to show: channel posts, or a sender
/// that is the chat already named on the line.
fn sender_label(m: &MessageInfo) -> Option<String> {
    if m.sender_id != 0 && m.chat_id == Some(m.sender_id) {
        return None;
    }
    match (&m.sender_name, m.sender_id) {
        (Some(n), _) => Some(n.clone()),
        (None, 0) => None,
        (None, id) => Some(id.to_string()),
    }
}

impl Render for MessageInfo {
    fn render(&self, full: bool) -> String {
        let mut s = format!("[{}] {}", self.id, format_date(self.date));
        if let Some(title) = &self.chat_title {
            s.push_str(&format!(" in {} ({})", title, self.chat_id.unwrap_or_default()));
        } else if let Some(id) = self.chat_id {
            s.push_str(&format!(" in {id}"));
        }
        if let Some(sender) = sender_label(self) {
            s.push(' ');
            s.push_str(&sender);
        }
        s.push(':');
        if let Some(r) = self.reply_to_msg_id {
            s.push_str(&format!(" ↩{r}"));
        }
        if let Some(media) = &self.media {
            s.push_str(&format!(" [{media}]"));
        }
        if !self.text.is_empty() {
            s.push(' ');
            s.push_str(&message_text(&self.text, full));
        }
        s
    }

    /// Lines plus a single legend mapping sender names to ids, so ids are
    /// available without repeating them on every line.
    fn render_list(items: &[Self], full: bool) -> String {
        let mut out = items.iter().map(|m| m.render(full)).collect::<Vec<_>>().join("\n");
        let senders: BTreeMap<&str, i64> = items
            .iter()
            .filter(|m| m.sender_id != 0 && m.chat_id != Some(m.sender_id))
            .filter_map(|m| m.sender_name.as_deref().map(|n| (n, m.sender_id)))
            .collect();
        if !senders.is_empty() {
            let legend = senders.iter().map(|(n, id)| format!("{n}={id}")).collect::<Vec<_>>().join(", ");
            out.push_str(&format!("\nsenders: {legend}"));
        }
        out
    }
}

/// Successful MCP tool result with rendered text.
pub fn ok<T: Render + ?Sized>(value: &T, full: bool) -> Value {
    json!({ "content": [{ "type": "text", "text": value.render(full) }] })
}

/// Tool-level error result (shown to the LLM, not a protocol error).
pub fn err(e: impl std::fmt::Display) -> Value {
    json!({ "content": [{ "type": "text", "text": format!("Error: {e}") }], "isError": true })
}

/// Reads the optional `full` flag shared by read tools.
pub fn full_flag(args: &Value) -> bool {
    args.get("full").and_then(|v| v.as_bool()).unwrap_or(false)
}

/// Schema snippet for the shared `full` flag.
pub fn full_schema() -> Value {
    json!({ "type": "boolean", "description": "Untruncated texts / extra fields" })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(id: i32, sender: Option<&str>, text: &str) -> MessageInfo {
        MessageInfo { id, sender_id: 42, sender_name: sender.map(str::to_string), text: text.into(), date: 0, ..Default::default() }
    }

    #[test]
    fn truncates_long_text_unless_full() {
        let long = "a".repeat(TRUNCATE_AT + 50);
        let m = msg(1, Some("Ann"), &long);
        assert!(m.render(false).ends_with("…[+50 chars]"));
        assert!(m.render(true).ends_with(&long));
    }

    #[test]
    fn message_list_has_sender_legend() {
        let out = vec![msg(1, Some("Ann"), "hi"), msg(2, Some("Ann"), "yo")].render(false);
        assert_eq!(out.lines().count(), 3);
        assert!(out.ends_with("senders: Ann=42"));
        assert!(out.contains("Ann: hi"));
    }

    #[test]
    fn channel_post_and_own_chat_have_no_sender() {
        let post = MessageInfo { id: 7, sender_id: 0, text: "news".into(), ..Default::default() };
        assert!(post.render(false).ends_with(": news"));
        assert!(!post.render(false).contains("?"));
        let global = MessageInfo { id: 8, sender_id: -100, sender_name: Some("Chan".into()), chat_id: Some(-100), chat_title: Some("Chan".into()), text: "x".into(), ..Default::default() };
        assert_eq!(global.render(false).matches("Chan").count(), 1);
        assert_eq!(vec![global].render(false).lines().count(), 1);
    }

    #[test]
    fn chat_line_is_compact() {
        let c = ChatInfo { id: -5, title: "Fam".into(), kind: "group".into(), members: Some(5), unread: Some(2), ..Default::default() };
        assert_eq!(c.render(false), "-5 | group | Fam | 5 members | 2 unread");
    }

    #[test]
    fn unit_and_empty_list() {
        assert_eq!(().render(false), "OK");
        assert_eq!(Vec::<ChatInfo>::new().render(false), "(none)");
    }

    #[test]
    fn contact_phone_only_when_full() {
        let c = ContactInfo { user_id: 1, first_name: "A".into(), last_name: None, username: None, phone: Some("44".into()) };
        assert!(!c.render(false).contains("44"));
        assert!(c.render(true).contains("+44"));
    }
}
