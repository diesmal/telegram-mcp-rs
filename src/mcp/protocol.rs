use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_request() {
        let json = r#"{"jsonrpc": "2.0", "method": "subtract", "params": [42, 23], "id": 1}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "subtract");
        assert_eq!(req.id, Some(Value::Number(1.into())));
        assert!(req.params.is_some());
    }

    #[test]
    fn test_deserialize_notification() {
        let json = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2,3]}"#;
        let req: JsonRpcRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "update");
        assert_eq!(req.id, None);
        assert!(req.params.is_some());
    }

    #[test]
    fn test_serialize_response() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: Some(Value::Number(19.into())),
            error: None,
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"jsonrpc":"2.0","id":1,"result":19}"#);
    }

    #[test]
    fn test_serialize_error() {
        let resp = JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id: Value::Number(1.into()),
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: "Method not found".to_string(),
                data: None,
            }),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert_eq!(json, r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}"#);
    }
}
