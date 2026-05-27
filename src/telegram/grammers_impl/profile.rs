use crate::telegram::ProfileService;
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl ProfileService for GrammersService {
    async fn update_profile(&self, first_name: Option<&str>, last_name: Option<&str>, about: Option<&str>) -> Result<()> {
        self.client.invoke(&tl::functions::account::UpdateProfile {
            first_name: first_name.map(|s| s.to_string()),
            last_name: last_name.map(|s| s.to_string()),
            about: about.map(|s| s.to_string()),
        }).await?;
        Ok(())
    }

    async fn set_profile_photo(&self, path: &str) -> Result<()> {
        let path_obj = self.path_guard.canonicalize_path(path, false)?;
        let uploaded_file = self.client.upload_file(&path_obj).await?;
        
        self.client.invoke(&tl::functions::photos::UploadProfilePhoto {
            fallback: false,
            file: Some(uploaded_file.raw),
            video: None,
            video_start_ts: None,
            video_emoji_markup: None,
            bot: None,
        }).await?;
        Ok(())
    }

    async fn delete_profile_photo(&self) -> Result<()> {
        let res = self.client.invoke(&tl::functions::photos::GetUserPhotos {
            user_id: tl::enums::InputUser::UserSelf,
            offset: 0,
            max_id: 0,
            limit: 1,
        }).await?;
        
        if let tl::enums::photos::Photos::Photos(p) = res {
            if let Some(photo) = p.photos.first() {
                if let tl::enums::Photo::Photo(ph) = photo {
                    self.client.invoke(&tl::functions::photos::DeletePhotos {
                        id: vec![tl::enums::InputPhoto::Photo(tl::types::InputPhoto {
                            id: ph.id,
                            access_hash: ph.access_hash,
                            file_reference: ph.file_reference.clone(),
                        })]
                    }).await?;
                }
            }
        }
        Ok(())
    }

    async fn get_user_status(&self, user_id: i64) -> Result<String> {
        let peer_ref = self.get_peer_ref(user_id).await?;
        let user = match peer_ref {
            tl::enums::InputPeer::User(u) => tl::enums::InputUser::User(tl::types::InputUser { user_id: u.user_id, access_hash: u.access_hash }),
            _ => anyhow::bail!("Must be a user"),
        };
        let res = self.client.invoke(&tl::functions::users::GetFullUser {
            id: user,
        }).await?;
        
        if let tl::enums::users::UserFull::Full(f) = res {
            for user in f.users {
                if let tl::enums::User::User(u) = user {
                    if let Some(status) = u.status {
                        return Ok(format!("{:?}", status));
                    }
                }
            }
        }
        Ok("Unknown".to_string())
    }

    async fn resolve_username(&self, username: &str) -> Result<i64> {
        // Technically mapped in AuthService::resolve_peer, just route to it.
        // Wait, the client is in GrammersService, so we can just do:
        let peer = self.client.resolve_username(username).await?.context("Peer not found")?;
        Ok(peer.id().bot_api_dialog_id())
    }
}
