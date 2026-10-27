use crate::telegram::{MediaService, MessageInfo};
use super::{convert, GrammersService};
use grammers_client::media::Downloadable;
use grammers_tl_types as tl;
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
        Ok(convert::msg_from_client(&msg))
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
        
        Ok(msgs.iter().map(convert::msg_from_client).collect())
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

        // grammers picks the photo size with the most bytes, which for small
        // photos is the inline stripped thumb that the server refuses to serve;
        // download the largest real size instead.
        let photo_location = match &msg.raw {
            tl::enums::Message::Message(m) => match &m.media {
                Some(tl::enums::MessageMedia::Photo(p)) => match &p.photo {
                    Some(tl::enums::Photo::Photo(photo)) => largest_photo_location(photo),
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        };
        let result = match photo_location {
            Some(loc) => self.save_download(&RawLocation(loc), &target_path).await,
            None => self.save_download(&media, &target_path).await,
        };
        if let Err(e) = result {
            // Don't leave an empty file behind.
            let _ = std::fs::remove_file(&target_path);
            return Err(e.into());
        }
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
        
        let label = match &msg.raw {
            grammers_tl_types::enums::Message::Message(m) => m.media.as_ref().and_then(convert::media_label),
            _ => None,
        };
        Ok(label.unwrap_or_else(|| "No media".to_string()))
    }

    async fn get_sticker_sets(&self) -> Result<Vec<String>> {
        Ok(vec![])
    }

    async fn get_user_photos(&self, _user_id: i64) -> Result<Vec<String>> {
        anyhow::bail!("User photos require MTProto v2 mapping")
    }
}

struct RawLocation(tl::enums::InputFileLocation);

impl Downloadable for RawLocation {
    fn to_raw_input_location(&self) -> Option<tl::enums::InputFileLocation> {
        Some(self.0.clone())
    }
}

/// Largest downloadable size of a photo by pixel area (ignores inline
/// stripped / cached thumbs).
fn largest_photo_location(photo: &tl::types::Photo) -> Option<tl::enums::InputFileLocation> {
    let thumb = photo
        .sizes
        .iter()
        .filter_map(|s| match s {
            tl::enums::PhotoSize::Size(s) => Some((s.w * s.h, s.r#type.clone())),
            tl::enums::PhotoSize::Progressive(s) => Some((s.w * s.h, s.r#type.clone())),
            _ => None,
        })
        .max_by_key(|(area, _)| *area)?
        .1;
    Some(tl::enums::InputFileLocation::InputPhotoFileLocation(tl::types::InputPhotoFileLocation {
        id: photo.id,
        access_hash: photo.access_hash,
        file_reference: photo.file_reference.clone(),
        thumb_size: thumb,
    }))
}

impl GrammersService {
    /// Like `Client::download_media`, but flushes the file before returning.
    /// grammers drops its tokio File unflushed, so an immediate read (get_image)
    /// could see an empty file.
    async fn save_download<D: Downloadable>(&self, item: &D, path: &std::path::Path) -> Result<()> {
        use tokio::io::AsyncWriteExt;
        let mut file = tokio::fs::File::create(path).await?;
        let mut download = self.client.iter_download(item);
        while let Some(chunk) = download.next().await? {
            file.write_all(&chunk).await?;
        }
        file.flush().await?;
        Ok(())
    }
}
