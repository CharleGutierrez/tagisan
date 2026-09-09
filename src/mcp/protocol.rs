use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Polymorphic JSON-RPC 2.0 identifier (String, Number, or Null)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RequestId {
    Number(i64),
    String(String),
}

impl std::fmt::Display for RequestId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RequestId::Number(n) => write!(f, "{n}"),
            RequestId::String(s) => write!(f, "{s}"),
        }
    }
}

impl From<u64> for RequestId {
    fn from(n: u64) -> Self {
        RequestId::Number(n as i64)
    }
}

impl From<i64> for RequestId {
    fn from(n: i64) -> Self {
        RequestId::Number(n)
    }
}

impl From<i32> for RequestId {
    fn from(n: i32) -> Self {
        RequestId::Number(n as i64)
    }
}

impl From<String> for RequestId {
    fn from(s: String) -> Self {
        RequestId::String(s)
    }
}

impl From<&str> for RequestId {
    fn from(s: &str) -> Self {
        RequestId::String(s.to_string())
    }
}

impl PartialEq<u64> for RequestId {
    fn eq(&self, other: &u64) -> bool {
        match self {
            RequestId::Number(n) => *n >= 0 && (*n as u64) == *other,
            _ => false,
        }
    }
}

impl PartialEq<RequestId> for u64 {
    fn eq(&self, other: &RequestId) -> bool {
        other == self
    }
}

impl PartialEq<i64> for RequestId {
    fn eq(&self, other: &i64) -> bool {
        match self {
            RequestId::Number(n) => *n == *other,
            _ => false,
        }
    }
}

impl PartialEq<&str> for RequestId {
    fn eq(&self, other: &&str) -> bool {
        match self {
            RequestId::String(s) => s == *other,
            _ => false,
        }
    }
}

/// Standard JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: RequestId,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: impl Into<RequestId>, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.into(),
            params,
        }
    }
}

/// Standard JSON-RPC 2.0 Notification (no id, no response expected)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcNotification {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcNotification {
    pub fn new(method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            method: method.into(),
            params,
        }
    }
}

/// Standard JSON-RPC 2.0 Response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(default)]
    pub id: Option<RequestId>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: impl Into<RequestId>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id.into()),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<RequestId>, code: i64, message: impl Into<String>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code,
                message: message.into(),
                data: None,
            }),
        }
    }
}

/// Standard JSON-RPC 2.0 Error object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(default)]
    pub data: Option<Value>,
}

impl std::fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "JSON-RPC error [{}]: {}", self.code, self.message)
    }
}

impl std::error::Error for JsonRpcError {}

// =========================================================================
// Model Context Protocol (MCP) Domain Data Structures
// =========================================================================

/// MCP Client Information passed during initialize handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpClientInfo {
    pub name: String,
    pub version: String,
}

/// MCP Server Information returned during initialize handshake
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerInfo {
    pub name: String,
    #[serde(default)]
    pub version: Option<String>,
}

/// Result returned by the MCP `initialize` method
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpInitializeResult {
    #[serde(rename = "protocolVersion")]
    pub protocol_version: String,
    #[serde(default)]
    pub capabilities: Value,
    #[serde(rename = "serverInfo", default)]
    pub server_info: Option<McpServerInfo>,
}

/// Result of an MCP `tools/list` query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolListResult {
    #[serde(default)]
    pub tools: Vec<McpToolDefinition>,
}

/// Definition of an individual tool published by an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct McpToolDefinition {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "inputSchema")]
    pub input_schema: Value,
}

/// Result of an MCP `tools/call` invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCallResult {
    #[serde(default)]
    pub content: Vec<McpContentBlock>,
    #[serde(rename = "isError", default)]
    pub is_error: bool,
}

impl McpToolCallResult {
    /// Extract combined text output from all content blocks (including text, images, and resources)
    pub fn extract_text(&self) -> String {
        let mut texts = Vec::new();
        for block in &self.content {
            match block {
                McpContentBlock::Text { text } => {
                    texts.push(text.clone());
                }
                McpContentBlock::Image { data, mime_type } => {
                    texts.push(format!("[Image: data:{};base64,{}]", mime_type, data));
                }
                McpContentBlock::Resource { resource } => {
                    if let Some(text) = resource.get("text").and_then(|v| v.as_str()) {
                        texts.push(text.to_string());
                    } else if let Some(blob) = resource.get("blob").and_then(|v| v.as_str()) {
                        let mime = resource.get("mimeType").and_then(|v| v.as_str()).unwrap_or("application/octet-stream");
                        texts.push(format!("[Blob resource: data:{};base64,{}]", mime, blob));
                    } else {
                        texts.push(serde_json::to_string(resource).unwrap_or_else(|_| resource.to_string()));
                    }
                }
                McpContentBlock::Unknown => {}
            }
        }
        if texts.is_empty() {
            String::new()
        } else {
            texts.join("\n")
        }
    }
}

/// Content block returned by an MCP tool invocation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum McpContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "resource")]
    Resource {
        resource: Value,
    },
    #[serde(other)]
    Unknown,
}
