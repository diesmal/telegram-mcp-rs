use crate::mcp::tool::Tool;

pub mod profile;
pub mod bots;
pub mod chats;
pub mod messages;
pub mod admin;
pub mod media;
pub mod folders;
pub mod contacts;
pub mod search;

/// Tool groups. `core` tools are always listed; the rest are listed only in
/// `TELEGRAM_TOOLS=all` mode and are otherwise discovered through the
/// `telegram_tools` / `telegram_call` meta tools.
pub const GROUPS: &[(&str, &str)] = &[
    ("core", "Everyday reading and messaging"),
    ("messages", "Pin/unpin, delete, links, reactions list, drafts, scheduled, delete history"),
    ("media", "Albums, media info"),
    ("chats", "Create/leave/join chats, mute, archive, rename, invite links, public chat search"),
    ("admin", "Ban/unban, promote/demote, rights, permissions, slow mode, admin/banned lists, invites"),
    ("contacts", "Contact list, add/delete, block/unblock, user status"),
    ("folders", "Chat folders"),
    ("profile", "Own profile and bots: profile edits, photos, bot info, inline buttons, bot commands"),
];

/// Every tool with its group. Stubs that only ever returned "not implemented"
/// and aliases of send_file are intentionally not registered.
pub fn registry() -> Vec<(&'static str, Box<dyn Tool>)> {
    vec![
        // core
        ("core", Box::new(profile::GetMeTool)),
        ("core", Box::new(chats::ListChatsTool)),
        ("core", Box::new(chats::GetChatInfoTool)),
        ("core", Box::new(messages::GetHistoryTool)),
        ("core", Box::new(messages::GetMessageContextTool)),
        ("core", Box::new(messages::SearchMessagesTool)),
        ("core", Box::new(messages::SearchGlobalMessagesTool)),
        ("core", Box::new(messages::SendMessageTool)),
        ("core", Box::new(messages::EditMessageTool)),
        ("core", Box::new(messages::ForwardMessagesTool)),
        ("core", Box::new(messages::SendReactionTool)),
        ("core", Box::new(messages::MarkAsReadTool)),
        ("core", Box::new(profile::ResolveUsernameTool)),
        ("core", Box::new(contacts::SearchContactsTool)),
        ("core", Box::new(admin::GetParticipantsTool)),
        ("core", Box::new(media::SendFileTool)),
        ("core", Box::new(media::DownloadMediaTool)),
        ("core", Box::new(media::GetImageTool)),
        // messages
        ("messages", Box::new(messages::DeleteMessagesTool)),
        ("messages", Box::new(messages::PinMessageTool)),
        ("messages", Box::new(messages::UnpinMessageTool)),
        ("messages", Box::new(messages::UnpinAllMessagesTool)),
        ("messages", Box::new(messages::GetPinnedMessagesTool)),
        ("messages", Box::new(messages::GetMessageLinkTool)),
        ("messages", Box::new(messages::GetMessageReactionsTool)),
        ("messages", Box::new(messages::RemoveReactionTool)),
        ("messages", Box::new(messages::GetScheduledMessagesTool)),
        ("messages", Box::new(messages::DeleteScheduledMessageTool)),
        ("messages", Box::new(messages::SaveDraftTool)),
        ("messages", Box::new(messages::ClearDraftsTool)),
        ("messages", Box::new(messages::DeleteChatHistoryTool)),
        // media
        ("media", Box::new(media::SendAlbumTool)),
        ("media", Box::new(media::GetMediaInfoTool)),
        // chats
        ("chats", Box::new(chats::CreateGroupTool)),
        ("chats", Box::new(chats::CreateChannelTool)),
        ("chats", Box::new(chats::LeaveChatTool)),
        ("chats", Box::new(chats::SetChatMutedTool)),
        ("chats", Box::new(chats::SetChatArchivedTool)),
        ("chats", Box::new(chats::EditChatTitleTool)),
        ("chats", Box::new(chats::EditChatAboutTool)),
        ("chats", Box::new(admin::GetInviteLinkTool)),
        ("chats", Box::new(admin::ImportChatInviteTool)),
        ("chats", Box::new(admin::JoinChatByLinkTool)),
        ("chats", Box::new(search::SearchPublicChatsTool)),
        // admin
        ("admin", Box::new(admin::BanUserTool)),
        ("admin", Box::new(admin::UnbanUserTool)),
        ("admin", Box::new(admin::PromoteAdminTool)),
        ("admin", Box::new(admin::DemoteAdminTool)),
        ("admin", Box::new(admin::EditAdminRightsTool)),
        ("admin", Box::new(admin::SetDefaultChatPermissionsTool)),
        ("admin", Box::new(admin::ToggleSlowModeTool)),
        ("admin", Box::new(admin::GetAdminsTool)),
        ("admin", Box::new(admin::GetBannedUsersTool)),
        ("admin", Box::new(admin::InviteUsersTool)),
        // contacts
        ("contacts", Box::new(contacts::ListContactsTool)),
        ("contacts", Box::new(contacts::AddContactTool)),
        ("contacts", Box::new(contacts::DeleteContactTool)),
        ("contacts", Box::new(contacts::BlockUserTool)),
        ("contacts", Box::new(contacts::UnblockUserTool)),
        ("contacts", Box::new(contacts::GetBlockedUsersTool)),
        ("contacts", Box::new(contacts::ExportContactsTool)),
        ("contacts", Box::new(profile::GetUserStatusTool)),
        // folders
        ("folders", Box::new(folders::ListFoldersTool)),
        ("folders", Box::new(folders::GetFolderTool)),
        ("folders", Box::new(folders::DeleteFolderTool)),
        ("folders", Box::new(folders::AddChatToFolderTool)),
        ("folders", Box::new(folders::RemoveChatFromFolderTool)),
        ("folders", Box::new(folders::ReorderFoldersTool)),
        // profile & bots
        ("profile", Box::new(profile::UpdateProfileTool)),
        ("profile", Box::new(profile::SetProfilePhotoTool)),
        ("profile", Box::new(profile::DeleteProfilePhotoTool)),
        ("profile", Box::new(bots::GetBotInfoTool)),
        ("profile", Box::new(bots::ListInlineButtonsTool)),
        ("profile", Box::new(bots::PressInlineButtonTool)),
        ("profile", Box::new(bots::SetBotCommandsTool)),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn names_unique_and_groups_known() {
        let groups: HashSet<_> = GROUPS.iter().map(|(g, _)| *g).collect();
        let mut names = HashSet::new();
        for (group, tool) in registry() {
            assert!(groups.contains(group), "unknown group {group}");
            assert!(names.insert(tool.info().name), "duplicate tool");
        }
    }
}
