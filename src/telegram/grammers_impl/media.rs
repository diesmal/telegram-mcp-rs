use crate::telegram::{MediaService, MessageInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use std::path::PathBuf;
use grammers_client::message::InputMessage;
use tracing::info;

#[async_trait]
impl MediaService for GrammersService {
    async fn send_file(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let path_obj = self.path_guard.canonicalize_path(path, false)?;
        
        let is_image = match path_obj.extension().and_then(|s| s.to_str()) {
            Some(ext) => ["jpg", "jpeg", "png"].contains(&ext.to_lowercase().as_str()),
            None => false,
        };

        let uploaded_file = self.client.upload_file(&path_obj).await?;
        
        let mut input = InputMessage::new().text(caption.unwrap_or(""));
        if is_image && path_obj.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase() != "png" {
            input = input.photo(uploaded_file);
        } else {
            input = input.document(uploaded_file);
        };

        let msg = self.client.send_message(peer_ref, input).await?;
        Ok(MessageInfo {
            id: msg.id(),
            sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
            text: msg.text().to_string(),
            date: msg.date().timestamp(),
            reply_to_msg_id: msg.reply_to_message_id(),
        })
    }

    async fn send_album(&self, peer_id: i64, paths: Vec<String>, caption: Option<&str>) -> Result<Vec<MessageInfo>> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let mut msgs = Vec::new();
        
        for (i, path) in paths.iter().enumerate() {
            let path_obj = self.path_guard.canonicalize_path(path, false)?;
            let uploaded_file = self.client.upload_file(&path_obj).await?;
            
            let is_image = match path_obj.extension().and_then(|s| s.to_str()) {
                Some(ext) => ["jpg", "jpeg", "png"].contains(&ext.to_lowercase().as_str()),
                None => false,
            };
            
            let mut input = InputMessage::new().text(if i == 0 { caption.unwrap_or("") } else { "" });
            if is_image && path_obj.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase() != "png" {
                input = input.photo(uploaded_file);
            } else {
                input = input.document(uploaded_file);
            };
            msgs.push(self.client.send_message(peer_ref.clone(), input).await?);
        }
        
        Ok(msgs.into_iter().map(|msg| MessageInfo {
            id: msg.id(),
            sender_id: msg.sender().map(|s| self.to_i64(s.id())).unwrap_or(0),
            text: msg.text().to_string(),
            date: msg.date().timestamp(),
            reply_to_msg_id: msg.reply_to_message_id(),
        }).collect())
    }

    async fn download_media(&self, peer_id: i64, message_id: i32, path: Option<&str>) -> Result<PathBuf> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let msg = self.client.get_messages_by_id(peer_ref, &[message_id]).await?
            .pop().flatten().context("Message not found")?;
        
        let media = msg.media().context("Message has no media")?;
        
        let target_path = if let Some(p) = path {
            self.path_guard.canonicalize_path(p, true)?
        } else {
            let downloads_dir = PathBuf::from("/home/di/telegram-mcp/downloads");
            if !downloads_dir.exists() {
                std::fs::create_dir_all(&downloads_dir)?;
            }
            let filename = format!("telegram_{}_{}_{}.media", peer_id, message_id, rand::random::<u32>());
            downloads_dir.join(filename)
        };

        self.client.download_media(&media, &target_path).await?;
        Ok(target_path)
    }

    async fn send_contact(&self, _peer_id: i64, _phone: &str, _first_name: &str, _last_name: &str) -> Result<MessageInfo> {
        anyhow::bail!("Contact requires InputMediaContact mapping")
    }

    async fn send_gif(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo> {
        self.send_file(peer_id, path, caption).await
    }

    async fn send_sticker(&self, peer_id: i64, path: &str) -> Result<MessageInfo> {
        self.send_file(peer_id, path, None).await
    }

    async fn send_voice(&self, peer_id: i64, path: &str, caption: Option<&str>) -> Result<MessageInfo> {
        self.send_file(peer_id, path, caption).await
    }

    async fn get_gif_search(&self, _query: &str) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_media_info(&self, peer_id: i64, message_id: i32) -> Result<String> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        let msg = self.client.get_messages_by_id(peer_ref, &[message_id]).await?
            .pop().flatten().context("Message not found")?;
        
        if let Some(media) = msg.media() {
            return Ok(format!("{:?}", media));
        }
        Ok("No media".to_string())
    }

    async fn get_sticker_sets(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_user_photos(&self, _user_id: i64) -> Result<Vec<String>> {
        anyhow::bail!("User photos require MTProto v2 mapping")
    }
}
