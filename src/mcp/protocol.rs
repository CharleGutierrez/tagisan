use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Standard JSON-RPC 2.0 Request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: u64, method: impl Into<String>, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
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
    pub id: Option<u64>,
    #[serde(default)]
    pub result: Option<Value>,
    #[serde(default)]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    pub fn success(id: u64, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: Some(id),
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<u64>, code: i64, message: impl Into<String>) -> Self {
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
