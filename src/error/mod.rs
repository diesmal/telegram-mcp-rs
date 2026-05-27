use thiserror::Error;

#[derive(Error, Debug)]
pub enum McpError {
    #[error("Parse error")]
    ParseError,
    #[error("Invalid request")]
    InvalidRequest,
    #[error("Method not found")]
    MethodNotFound,
    #[error("Invalid params")]
    InvalidParams,
    #[error("Internal error")]
    InternalError,
    #[error("Custom error: {0}")]
    Custom(String),
}

impl McpError {
    pub fn to_code(&self) -> i32 {
        match self {
            McpError::ParseError => -32700,
            McpError::InvalidRequest => -32600,
            McpError::MethodNotFound => -32601,
            McpError::InvalidParams => -32602,
            McpError::InternalError => -32603,
            McpError::Custom(_) => -32000,
        }
    }
}
