use crate::mcp::tool::Tool;
use std::collections::HashMap;

pub mod profile;
pub mod bots;
pub mod chats;
pub mod messages;
pub mod admin;
pub mod media;
pub mod folders;
pub mod contacts;

pub fn register_all_tools() -> HashMap<String, Box<dyn Tool>> {
    let mut registry: HashMap<String, Box<dyn Tool>> = HashMap::new();
    
    // Profile tools
    let get_me = profile::GetMeTool {};
    registry.insert(get_me.info().name, Box::new(get_me));
    
    let update_profile = profile::UpdateProfileTool {};
    registry.insert(update_profile.info().name, Box::new(update_profile));
    
    let set_profile_photo = profile::SetProfilePhotoTool {};
    registry.insert(set_profile_photo.info().name, Box::new(set_profile_photo));

    // Bot tools
    let get_bot_info = bots::GetBotInfoTool {};
    registry.insert(get_bot_info.info().name, Box::new(get_bot_info));
    
    let list_inline_buttons = bots::ListInlineButtonsTool {};
    registry.insert(list_inline_buttons.info().name, Box::new(list_inline_buttons));
    
    let press_inline_button = bots::PressInlineButtonTool {};
    registry.insert(press_inline_button.info().name, Box::new(press_inline_button));

    // Chat tools
    let list_chats = chats::ListChatsTool {};
    registry.insert(list_chats.info().name, Box::new(list_chats));
    
    let get_chat_info = chats::GetChatInfoTool {};
    registry.insert(get_chat_info.info().name, Box::new(get_chat_info));
    
    let create_group = chats::CreateGroupTool {};
    registry.insert(create_group.info().name, Box::new(create_group));
    
    let create_channel = chats::CreateChannelTool {};
    registry.insert(create_channel.info().name, Box::new(create_channel));
    
    let leave_chat = chats::LeaveChatTool {};
    registry.insert(leave_chat.info().name, Box::new(leave_chat));
    
    let set_chat_muted = chats::SetChatMutedTool {};
    registry.insert(set_chat_muted.info().name, Box::new(set_chat_muted));
    
    let set_chat_archived = chats::SetChatArchivedTool {};
    registry.insert(set_chat_archived.info().name, Box::new(set_chat_archived));

    // Messages tools
    let get_messages = messages::GetMessagesTool {};
    registry.insert(get_messages.info().name, Box::new(get_messages));
    
    let get_history = messages::GetHistoryTool {};
    registry.insert(get_history.info().name, Box::new(get_history));
    
    let search_messages = messages::SearchMessagesTool {};
    registry.insert(search_messages.info().name, Box::new(search_messages));
    
    let send_message = messages::SendMessageTool {};
    registry.insert(send_message.info().name, Box::new(send_message));
    
    let edit_message = messages::EditMessageTool {};
    registry.insert(edit_message.info().name, Box::new(edit_message));
    
    let delete_messages = messages::DeleteMessagesTool {};
    registry.insert(delete_messages.info().name, Box::new(delete_messages));
    
    let forward_messages = messages::ForwardMessagesTool {};
    registry.insert(forward_messages.info().name, Box::new(forward_messages));
    
    let send_reaction = messages::SendReactionTool {};
    registry.insert(send_reaction.info().name, Box::new(send_reaction));
    
    let get_scheduled_messages = messages::GetScheduledMessagesTool {};
    registry.insert(get_scheduled_messages.info().name, Box::new(get_scheduled_messages));
    
    let save_draft = messages::SaveDraftTool {};
    registry.insert(save_draft.info().name, Box::new(save_draft));
    
    let clear_drafts = messages::ClearDraftsTool {};
    registry.insert(clear_drafts.info().name, Box::new(clear_drafts));

    // Admin tools
    let ban_user = admin::BanUserTool {};
    registry.insert(ban_user.info().name, Box::new(ban_user));
    
    let promote_admin = admin::PromoteAdminTool {};
    registry.insert(promote_admin.info().name, Box::new(promote_admin));
    
    let get_participants = admin::GetParticipantsTool {};
    registry.insert(get_participants.info().name, Box::new(get_participants));
    
    let invite_users = admin::InviteUsersTool {};
    registry.insert(invite_users.info().name, Box::new(invite_users));

    // Media tools
    let send_file = media::SendFileTool {};
    registry.insert(send_file.info().name, Box::new(send_file));
    
    let send_album = media::SendAlbumTool {};
    registry.insert(send_album.info().name, Box::new(send_album));
    
    let download_media = media::DownloadMediaTool {};
    registry.insert(download_media.info().name, Box::new(download_media));

    // Folder tools
    let list_folders = folders::ListFoldersTool {};
    registry.insert(list_folders.info().name, Box::new(list_folders));
    
    let get_folder = folders::GetFolderTool {};
    registry.insert(get_folder.info().name, Box::new(get_folder));

    // Contact tools
    let list_contacts = contacts::ListContactsTool {};
    registry.insert(list_contacts.info().name, Box::new(list_contacts));
    
    let search_contacts = contacts::SearchContactsTool {};
    registry.insert(search_contacts.info().name, Box::new(search_contacts));
    
    let add_contact = contacts::AddContactTool {};
    registry.insert(add_contact.info().name, Box::new(add_contact));
    
    let delete_contact = contacts::DeleteContactTool {};
    registry.insert(delete_contact.info().name, Box::new(delete_contact));
    
    let block_user = contacts::BlockUserTool {};
    registry.insert(block_user.info().name, Box::new(block_user));
    
    let unblock_user = contacts::UnblockUserTool {};
    registry.insert(unblock_user.info().name, Box::new(unblock_user));

        let create_folder_tool = folders::CreateFolderTool {};
    registry.insert(create_folder_tool.info().name, Box::new(create_folder_tool));

    let delete_folder_tool = folders::DeleteFolderTool {};
    registry.insert(delete_folder_tool.info().name, Box::new(delete_folder_tool));

    let add_chat_to_folder_tool = folders::AddChatToFolderTool {};
    registry.insert(add_chat_to_folder_tool.info().name, Box::new(add_chat_to_folder_tool));

    let remove_chat_from_folder_tool = folders::RemoveChatFromFolderTool {};
    registry.insert(remove_chat_from_folder_tool.info().name, Box::new(remove_chat_from_folder_tool));

    let reorder_folders_tool = folders::ReorderFoldersTool {};
    registry.insert(reorder_folders_tool.info().name, Box::new(reorder_folders_tool));

    let demote_admin_tool = admin::DemoteAdminTool {};
    registry.insert(demote_admin_tool.info().name, Box::new(demote_admin_tool));

    let edit_admin_rights_tool = admin::EditAdminRightsTool {};
    registry.insert(edit_admin_rights_tool.info().name, Box::new(edit_admin_rights_tool));

    let set_default_chat_permissions_tool = admin::SetDefaultChatPermissionsTool {};
    registry.insert(set_default_chat_permissions_tool.info().name, Box::new(set_default_chat_permissions_tool));

    let toggle_slow_mode_tool = admin::ToggleSlowModeTool {};
    registry.insert(toggle_slow_mode_tool.info().name, Box::new(toggle_slow_mode_tool));

    let get_admins_tool = admin::GetAdminsTool {};
    registry.insert(get_admins_tool.info().name, Box::new(get_admins_tool));

    let get_banned_users_tool = admin::GetBannedUsersTool {};
    registry.insert(get_banned_users_tool.info().name, Box::new(get_banned_users_tool));

    let get_recent_actions_tool = admin::GetRecentActionsTool {};
    registry.insert(get_recent_actions_tool.info().name, Box::new(get_recent_actions_tool));

    let unban_user_tool = admin::UnbanUserTool {};
    registry.insert(unban_user_tool.info().name, Box::new(unban_user_tool));

    let get_invite_link_tool = admin::GetInviteLinkTool {};
    registry.insert(get_invite_link_tool.info().name, Box::new(get_invite_link_tool));

    let export_chat_invite_tool = admin::ExportChatInviteTool {};
    registry.insert(export_chat_invite_tool.info().name, Box::new(export_chat_invite_tool));

    let import_chat_invite_tool = admin::ImportChatInviteTool {};
    registry.insert(import_chat_invite_tool.info().name, Box::new(import_chat_invite_tool));

    let join_chat_by_link_tool = admin::JoinChatByLinkTool {};
    registry.insert(join_chat_by_link_tool.info().name, Box::new(join_chat_by_link_tool));

    let edit_chat_about_tool = chats::EditChatAboutTool {};
    registry.insert(edit_chat_about_tool.info().name, Box::new(edit_chat_about_tool));

    let edit_chat_photo_tool = chats::EditChatPhotoTool {};
    registry.insert(edit_chat_photo_tool.info().name, Box::new(edit_chat_photo_tool));

    let edit_chat_title_tool = chats::EditChatTitleTool {};
    registry.insert(edit_chat_title_tool.info().name, Box::new(edit_chat_title_tool));

    let delete_chat_photo_tool = chats::DeleteChatPhotoTool {};
    registry.insert(delete_chat_photo_tool.info().name, Box::new(delete_chat_photo_tool));

    let create_poll_tool = messages::CreatePollTool {};
    registry.insert(create_poll_tool.info().name, Box::new(create_poll_tool));

    let get_message_context_tool = messages::GetMessageContextTool {};
    registry.insert(get_message_context_tool.info().name, Box::new(get_message_context_tool));

    let get_message_link_tool = messages::GetMessageLinkTool {};
    registry.insert(get_message_link_tool.info().name, Box::new(get_message_link_tool));

    let get_message_reactions_tool = messages::GetMessageReactionsTool {};
    registry.insert(get_message_reactions_tool.info().name, Box::new(get_message_reactions_tool));

    let get_message_read_by_tool = messages::GetMessageReadByTool {};
    registry.insert(get_message_read_by_tool.info().name, Box::new(get_message_read_by_tool));

    let get_pinned_messages_tool = messages::GetPinnedMessagesTool {};
    registry.insert(get_pinned_messages_tool.info().name, Box::new(get_pinned_messages_tool));

    let unpin_message_tool = messages::UnpinMessageTool {};
    registry.insert(unpin_message_tool.info().name, Box::new(unpin_message_tool));

    let unpin_all_messages_tool = messages::UnpinAllMessagesTool {};
    registry.insert(unpin_all_messages_tool.info().name, Box::new(unpin_all_messages_tool));

    let delete_scheduled_message_tool = messages::DeleteScheduledMessageTool {};
    registry.insert(delete_scheduled_message_tool.info().name, Box::new(delete_scheduled_message_tool));

    let remove_reaction_tool = messages::RemoveReactionTool {};
    registry.insert(remove_reaction_tool.info().name, Box::new(remove_reaction_tool));

    let delete_chat_history_tool = messages::DeleteChatHistoryTool {};
    registry.insert(delete_chat_history_tool.info().name, Box::new(delete_chat_history_tool));

    let send_contact_tool = media::SendContactTool {};
    registry.insert(send_contact_tool.info().name, Box::new(send_contact_tool));

    let send_gif_tool = media::SendGifTool {};
    registry.insert(send_gif_tool.info().name, Box::new(send_gif_tool));

    let send_sticker_tool = media::SendStickerTool {};
    registry.insert(send_sticker_tool.info().name, Box::new(send_sticker_tool));

    let send_voice_tool = media::SendVoiceTool {};
    registry.insert(send_voice_tool.info().name, Box::new(send_voice_tool));

    let get_gif_search_tool = media::GetGifSearchTool {};
    registry.insert(get_gif_search_tool.info().name, Box::new(get_gif_search_tool));

    let get_media_info_tool = media::GetMediaInfoTool {};
    registry.insert(get_media_info_tool.info().name, Box::new(get_media_info_tool));

    let get_sticker_sets_tool = media::GetStickerSetsTool {};
    registry.insert(get_sticker_sets_tool.info().name, Box::new(get_sticker_sets_tool));

    let get_user_photos_tool = media::GetUserPhotosTool {};
    registry.insert(get_user_photos_tool.info().name, Box::new(get_user_photos_tool));

    let delete_profile_photo_tool = profile::DeleteProfilePhotoTool {};
    registry.insert(delete_profile_photo_tool.info().name, Box::new(delete_profile_photo_tool));

    let get_user_status_tool = profile::GetUserStatusTool {};
    registry.insert(get_user_status_tool.info().name, Box::new(get_user_status_tool));

    let set_bot_commands_tool = bots::SetBotCommandsTool {};
    registry.insert(set_bot_commands_tool.info().name, Box::new(set_bot_commands_tool));

    let get_blocked_users_tool = contacts::GetBlockedUsersTool {};
    registry.insert(get_blocked_users_tool.info().name, Box::new(get_blocked_users_tool));

    let import_contacts_tool = contacts::ImportContactsTool {};
    registry.insert(import_contacts_tool.info().name, Box::new(import_contacts_tool));

    let export_contacts_tool = contacts::ExportContactsTool {};
    registry.insert(export_contacts_tool.info().name, Box::new(export_contacts_tool));

        let pin_message_tool = messages::PinMessageTool {};
    registry.insert(pin_message_tool.info().name, Box::new(pin_message_tool));

        let mark_as_read_tool = messages::MarkAsReadTool {};
    registry.insert(mark_as_read_tool.info().name, Box::new(mark_as_read_tool));

        let get_drafts_tool = messages::GetDraftsTool {};
    registry.insert(get_drafts_tool.info().name, Box::new(get_drafts_tool));

        let resolve_username_tool = profile::ResolveUsernameTool {};
    registry.insert(resolve_username_tool.info().name, Box::new(resolve_username_tool));

        let get_contact_ids_tool = contacts::GetContactIdsTool {};
    registry.insert(get_contact_ids_tool.info().name, Box::new(get_contact_ids_tool));

        let get_direct_chat_by_contact_tool = contacts::GetDirectChatByContactTool {};
    registry.insert(get_direct_chat_by_contact_tool.info().name, Box::new(get_direct_chat_by_contact_tool));

        let get_contact_chats_tool = contacts::GetContactChatsTool {};
    registry.insert(get_contact_chats_tool.info().name, Box::new(get_contact_chats_tool));

        let get_last_interaction_tool = contacts::GetLastInteractionTool {};
    registry.insert(get_last_interaction_tool.info().name, Box::new(get_last_interaction_tool));

        let list_accounts_tool = profile::ListAccountsTool {};
    registry.insert(list_accounts_tool.info().name, Box::new(list_accounts_tool));

        let wait_for_new_message_tool = messages::WaitForNewMessageTool {};
    registry.insert(wait_for_new_message_tool.info().name, Box::new(wait_for_new_message_tool));

        let wait_for_settled_message_tool = messages::WaitForSettledMessageTool {};
    registry.insert(wait_for_settled_message_tool.info().name, Box::new(wait_for_settled_message_tool));

    registry
}
