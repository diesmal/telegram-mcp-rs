use crate::mcp::protocol::JsonRpcError;
use crate::mcp::tool::{Tool, ToolInfo, invalid_params};
use crate::telegram::TelegramService;
use async_trait::async_trait;
use serde_json::Value;
use std::sync::Arc;

pub struct SendFileTool;

#[async_trait]
impl Tool for SendFileTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_file".to_string(),
            description: "Send a local file to a chat.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "path": { "type": "string" },
                    "caption": { "type": "string" }
                },
                "required": ["chat_id", "path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        let caption = args.get("caption").and_then(|v| v.as_str());
        match telegram.send_file(chat_id, path, caption).await {
            Ok(msg) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("File sent (ID: {})", msg.id) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendAlbumTool;

#[async_trait]
impl Tool for SendAlbumTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_album".to_string(),
            description: "Send multiple files as an album.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "paths": { "type": "array", "items": { "type": "string" } },
                    "caption": { "type": "string" }
                },
                "required": ["chat_id", "paths"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let paths = args.get("paths").and_then(|v| v.as_array())
            .ok_or_else(|| invalid_params("paths must be an array of strings"))?
            .iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<Vec<_>>();
        let caption = args.get("caption").and_then(|v| v.as_str());
        match telegram.send_album(chat_id, paths, caption).await {
            Ok(msgs) => {
                let ids: Vec<i32> = msgs.iter().map(|m| m.id).collect();
                Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Album sent (IDs: {:?})", ids) }] }))
            }
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct DownloadMediaTool;

#[async_trait]
impl Tool for DownloadMediaTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "download_media".to_string(),
            description: "Download media from a message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" },
                    "path": { "type": "string" }
                },
                "required": ["chat_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        let path = args.get("path").and_then(|v| v.as_str());
        match telegram.download_media(chat_id, message_id, path).await {
            Ok(file_path) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Media downloaded to: {:?}", file_path) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetImageTool;

#[async_trait]
impl Tool for GetImageTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_image".to_string(),
            description: "Get an image from a message as base64 data so the LLM can see it.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {
                    "chat_id": { "type": "integer" },
                    "message_id": { "type": "integer" }
                },
                "required": ["chat_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let chat_id = args.get("chat_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("chat_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        
        // Download to a temporary location
        match telegram.download_media(chat_id, message_id, None).await {
            Ok(file_path) => {
                // Read the file and encode to base64
                match std::fs::read(&file_path) {
                    Ok(bytes) => {
                        use base64::{Engine as _, engine::general_purpose};
                        let b64 = general_purpose::STANDARD.encode(&bytes);
                        
                        // Try to determine mime type from extension
                        let mime_type = match file_path.extension().and_then(|s| s.to_str()) {
                            Some("png") => "image/png",
                            Some("jpg") | Some("jpeg") => "image/jpeg",
                            Some("gif") => "image/gif",
                            Some("webp") => "image/webp",
                            _ => "image/jpeg", // Default
                        };

                        Ok(serde_json::json!({
                            "content": [
                                {
                                    "type": "image",
                                    "data": b64,
                                    "mimeType": mime_type
                                }
                            ]
                        }))
                    }
                    Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error reading downloaded file: {}", e) }], "isError": true })),
                }
            }
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error downloading media: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendContactTool;

#[async_trait]
impl Tool for SendContactTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_contact".to_string(),
            description: "Send contact.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "phone": { "type": "string" }, "first_name": { "type": "string" }, "last_name": { "type": "string" } }
                , "required": ["peer_id", "phone", "first_name", "last_name"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let phone = args.get("phone").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("phone is required"))?;
        let first_name = args.get("first_name").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("first_name is required"))?;
        let last_name = args.get("last_name").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("last_name is required"))?;
        match telegram.send_contact(peer_id, phone, first_name, last_name).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendGifTool;

#[async_trait]
impl Tool for SendGifTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_gif".to_string(),
            description: "Send GIF.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "path": { "type": "string" }, "caption": { "type": "string" } }
                , "required": ["peer_id", "path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        let caption = args.get("caption").and_then(|v| v.as_str());
        match telegram.send_gif(peer_id, path, caption).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendStickerTool;

#[async_trait]
impl Tool for SendStickerTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_sticker".to_string(),
            description: "Send sticker.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "path": { "type": "string" } }
                , "required": ["peer_id", "path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        match telegram.send_sticker(peer_id, path).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct SendVoiceTool;

#[async_trait]
impl Tool for SendVoiceTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "send_voice".to_string(),
            description: "Send voice message.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "path": { "type": "string" }, "caption": { "type": "string" } }
                , "required": ["peer_id", "path"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("path is required"))?;
        let caption = args.get("caption").and_then(|v| v.as_str());
        match telegram.send_voice(peer_id, path, caption).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetGifSearchTool;

#[async_trait]
impl Tool for GetGifSearchTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_gif_search".to_string(),
            description: "Search GIFs.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "query": { "type": "string" } }
                , "required": ["query"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let query = args.get("query").and_then(|v| v.as_str()).ok_or_else(|| invalid_params("query is required"))?;
        match telegram.get_gif_search(query).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetMediaInfoTool;

#[async_trait]
impl Tool for GetMediaInfoTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_media_info".to_string(),
            description: "Get media info.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "peer_id": { "type": "integer" }, "message_id": { "type": "integer" } }
                , "required": ["peer_id", "message_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let peer_id = args.get("peer_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("peer_id is required"))?;
        let message_id = args.get("message_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("message_id is required"))? as i32;
        match telegram.get_media_info(peer_id, message_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetStickerSetsTool;

#[async_trait]
impl Tool for GetStickerSetsTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_sticker_sets".to_string(),
            description: "Get sticker sets.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": {  }
                
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {

        match telegram.get_sticker_sets().await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}

pub struct GetUserPhotosTool;

#[async_trait]
impl Tool for GetUserPhotosTool {
    fn info(&self) -> ToolInfo {
        ToolInfo {
            name: "get_user_photos".to_string(),
            description: "Get user photos.".to_string(),
            input_schema: serde_json::json!({
                "type": "object",
                "properties": { "user_id": { "type": "integer" } }
                , "required": ["user_id"]
            }),
        }
    }

    async fn execute(&self, args: Value, telegram: Arc<dyn TelegramService>) -> Result<Value, JsonRpcError> {
        let user_id = args.get("user_id").and_then(|v| v.as_i64()).ok_or_else(|| invalid_params("user_id is required"))?;
        match telegram.get_user_photos(user_id).await {
            Ok(res) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": serde_json::to_string_pretty(&res).unwrap_or_else(|_| "Success".to_string()) }] })),
            Err(e) => Ok(serde_json::json!({ "content": [{ "type": "text", "text": format!("Error: {}", e) }], "isError": true })),
        }
    }
}
