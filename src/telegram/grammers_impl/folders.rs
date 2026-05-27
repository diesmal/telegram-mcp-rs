use crate::telegram::{FolderService, FolderInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl FolderService for GrammersService {
    async fn list_folders(&self) -> Result<Vec<FolderInfo>> {
        let _res = self.client.invoke(&tl::functions::messages::GetDialogFilters {}).await?;
        Ok(vec![])
    }

    async fn get_folder(&self, folder_id: i32) -> Result<FolderInfo> {
        let folders = self.list_folders().await?;
        folders.into_iter().find(|f| f.id == folder_id).context("Folder not found")
    }

    async fn create_folder(&self, _title: &str, _included_peers: Vec<i64>, _excluded_peers: Vec<i64>) -> Result<i32> {
        anyhow::bail!("Folder creation requires MTProto v2 mapping")
    }

    async fn delete_folder(&self, folder_id: i32) -> Result<()> {
        self.client.invoke(&tl::functions::messages::UpdateDialogFilter {
            id: folder_id,
            filter: None,
        }).await?;
        Ok(())
    }

    async fn add_chat_to_folder(&self, folder_id: i32, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::folders::EditPeerFolders {
            folder_peers: vec![tl::enums::InputFolderPeer::Peer(tl::types::InputFolderPeer {
                peer: peer_ref,
                folder_id,
            })],
        }).await?;
        Ok(())
    }

    async fn remove_chat_from_folder(&self, folder_id: i32, peer_id: i64) -> Result<()> {
        // Technically this involves removing from the explicit DialogFilter or sending to folder 0
        // We'll set folder_id to 0 for the peer.
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::folders::EditPeerFolders {
            folder_peers: vec![tl::enums::InputFolderPeer::Peer(tl::types::InputFolderPeer {
                peer: peer_ref,
                folder_id: 0,
            })],
        }).await?;
        Ok(())
    }

    async fn reorder_folders(&self, folder_ids: Vec<i32>) -> Result<()> {
        self.client.invoke(&tl::functions::messages::UpdateDialogFiltersOrder {
            order: folder_ids,
        }).await?;
        Ok(())
    }
}
