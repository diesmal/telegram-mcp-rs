use crate::telegram::{ContactService, ContactInfo, ChatInfo};
use super::GrammersService;
use anyhow::{Result, Context};
use async_trait::async_trait;
use grammers_tl_types as tl;

#[async_trait]
impl ContactService for GrammersService {
    async fn list_contacts(&self) -> Result<Vec<ContactInfo>> {
        let res = self.client.invoke(&tl::functions::contacts::GetContacts { hash: 0 }).await?;
        let mut result = Vec::new();
        if let tl::enums::contacts::Contacts::Contacts(c) = res {
            for user in c.users {
                if let tl::enums::User::User(u) = user {
                    result.push(ContactInfo {
                        user_id: u.id,
                        first_name: u.first_name.unwrap_or_default(),
                        last_name: u.last_name,
                        username: u.username,
                        phone: u.phone,
                    });
                }
            }
        }
        Ok(result)
    }

    async fn search_contacts(&self, query: &str) -> Result<Vec<ContactInfo>> {
        let res = self.client.invoke(&tl::functions::contacts::Search {
            q: query.to_string(),
            limit: 50,
        }).await?;
        
        let mut result = Vec::new();
        if let tl::enums::contacts::Found::Found(f) = res {
            for user_enum in f.users {
                if let tl::enums::User::User(u) = user_enum {
                    result.push(ContactInfo {
                        user_id: u.id,
                        first_name: u.first_name.unwrap_or_default(),
                        last_name: u.last_name,
                        username: u.username,
                        phone: u.phone,
                    });
                }
            }
        }
        Ok(result)
    }

    async fn add_contact(&self, user_id: i64, first_name: &str, last_name: &str, phone: &str) -> Result<()> {
        let user = tl::enums::InputUser::User(tl::types::InputUser { user_id, access_hash: 0 });
        self.client.invoke(&tl::functions::contacts::AddContact {
            add_phone_privacy_exception: false,
            id: user,
            first_name: first_name.to_string(),
            last_name: last_name.to_string(),
            phone: phone.to_string(),
            note: None,
        }).await?;
        Ok(())
    }

    async fn delete_contact(&self, user_id: i64) -> Result<()> {
        let user = tl::enums::InputUser::User(tl::types::InputUser { user_id, access_hash: 0 });
        self.client.invoke(&tl::functions::contacts::DeleteContacts {
            id: vec![user],
        }).await?;
        Ok(())
    }

    async fn block_user(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::contacts::Block {
            id: peer_ref,
            my_stories_from: false,
        }).await?;
        Ok(())
    }

    async fn unblock_user(&self, peer_id: i64) -> Result<()> {
        let peer_ref = self.get_peer_ref(peer_id).await?;
        self.client.invoke(&tl::functions::contacts::Unblock {
            id: peer_ref,
            my_stories_from: false,
        }).await?;
        Ok(())
    }

    async fn get_blocked_users(&self) -> Result<Vec<crate::telegram::UserInfo>> {
        let res = self.client.invoke(&tl::functions::contacts::GetBlocked {
            my_stories_from: false,
            offset: 0,
            limit: 50,
        }).await?;
        
        let mut result = Vec::new();
        if let tl::enums::contacts::Blocked::Blocked(b) = res {
            for user in b.users {
                if let tl::enums::User::User(u) = user {
                    result.push(crate::telegram::UserInfo {
                        id: u.id,
                        first_name: u.first_name.unwrap_or_default(),
                        last_name: u.last_name,
                        username: u.username,
                    });
                }
            }
        }
        Ok(result)
    }

    async fn import_contacts(&self, _contacts: Vec<(String, String, String)>) -> Result<()> {
        anyhow::bail!("Import contacts requires MTProto v2 mapping")
    }

    async fn export_contacts(&self) -> Result<String> {
        // Just call get_contacts and format it
        let contacts = self.list_contacts().await?;
        let mut out = String::new();
        for c in contacts {
            out.push_str(&format!("{} {} ({})\n", c.first_name, c.last_name.unwrap_or_default(), c.phone.unwrap_or_default()));
        }
        Ok(out)
    }

    async fn get_contact_ids(&self) -> Result<Vec<i64>> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }
    
    async fn get_direct_chat_by_contact(&self, _contact_query: &str) -> Result<i64> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }

    async fn get_contact_chats(&self, _contact_id: i64) -> Result<Vec<ChatInfo>> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }

    async fn get_last_interaction(&self, _contact_id: i64) -> Result<i32> {
        anyhow::bail!("Endpoint requires MTProto v2 mapping")
    }
}
