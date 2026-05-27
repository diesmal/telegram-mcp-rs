use crate::telegram::{BotService, BotInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl BotService for GrammersService {
    async fn get_bot_info(&self, bot_username: &str) -> Result<BotInfo> {
        let peer = self.client.resolve_username(bot_username).await?.context("Bot not found")?;
        let input_user: tl::enums::InputUser = tl::enums::InputUser::User(tl::types::InputUser { 
            user_id: peer.id().bot_api_dialog_id(), // Simplified
            access_hash: 0,
        });
        
        let info = self.client.invoke(&tl::functions::bots::GetBotInfo {
            bot: Some(input_user),
            lang_code: "".to_string(),
        }).await?;
        
        if let tl::enums::bots::BotInfo::Info(i) = info {
            return Ok(BotInfo {
                name: i.name,
                about: i.about,
                description: i.description,
            });
        }
        anyhow::bail!("Invalid response")
    }

    async fn list_inline_buttons(&self, peer_id: i64, message_id: i32) -> Result<Vec<String>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let msg = self.client.get_messages_by_id(peer_ref, &[message_id]).await?
            .pop().flatten().context("Message not found")?;
        
        let mut buttons = Vec::new();
        if let Some(markup) = msg.reply_markup() {
            if let tl::enums::ReplyMarkup::ReplyInlineMarkup(im) = markup {
                for row in im.rows {
                    if let tl::enums::KeyboardButtonRow::Row(r) = row {
                        for button in r.buttons {
                            if let tl::enums::KeyboardButton::Callback(cb) = button {
                                buttons.push(format!("Text: '{}' -> Data: '{}'", cb.text, String::from_utf8_lossy(&cb.data)));
                            }
                        }
                    }
                }
            }
        }
        Ok(buttons)
    }

    async fn press_inline_button(&self, peer_id: i64, message_id: i32, button_data: &str) -> Result<String> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let data = button_data.as_bytes().to_vec();
        
        let res = self.client.invoke(&tl::functions::messages::GetBotCallbackAnswer {
            game: false,
            peer: peer_ref,
            msg_id: message_id,
            data: Some(data),
            password: None,
        }).await?;
        
        if let tl::enums::messages::BotCallbackAnswer::Answer(a) = res {
            return Ok(a.message.unwrap_or_else(|| "Callback triggered successfully.".to_string()));
        }
        anyhow::bail!("Invalid response")
    }

    async fn set_bot_commands(&self, commands: Vec<(String, String)>) -> Result<()> {
        let mut bot_commands = Vec::new();
        for (command, description) in commands {
            bot_commands.push(tl::enums::BotCommand::Command(tl::types::BotCommand {
                command,
                description,
            }));
        }
        
        self.client.invoke(&tl::functions::bots::SetBotCommands {
            scope: tl::enums::BotCommandScope::Default,
            lang_code: "".to_string(),
            commands: bot_commands,
        }).await?;
        Ok(())
    }
}
