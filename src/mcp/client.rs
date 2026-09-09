use crate::error::{Result, TagisanError};
use crate::mcp::config::McpServerConfig;
use crate::mcp::protocol::{
    JsonRpcNotification, JsonRpcRequest, McpInitializeResult, McpServerInfo, McpToolCallResult,
    McpToolDefinition, McpToolListResult,
};
use crate::mcp::transport::StdioTransport;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tracing::info;

pub const DEFAULT_MCP_PROTOCOL_VERSION: &str = "2024-11-05";
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// High-level client representing an active connection to an MCP server
pub struct McpClient {
    pub server_name: String,
    transport: Arc<StdioTransport>,
    next_id: AtomicU64,
    pub server_info: Option<McpServerInfo>,
    pub protocol_version: String,
    pub default_timeout: Duration,
}

impl McpClient {
    /// Connect to an MCP server process, perform the standard initialize handshake, and return the client
    pub async fn connect(
        server_name: impl Into<String>,
        config: &McpServerConfig,
    ) -> Result<Self> {
        let name = server_name.into();
        info!("Spawning MCP server process '{}'...", name);

        let transport = Arc::new(StdioTransport::spawn(name.clone(), config).await?);
        let client = Self {
            server_name: name,
            transport,
            next_id: AtomicU64::new(1),
            server_info: None,
            protocol_version: DEFAULT_MCP_PROTOCOL_VERSION.to_string(),
            default_timeout: DEFAULT_TIMEOUT,
        };

        // Perform standard MCP initialization handshake
        client.initialize().await
    }

    /// Perform the initialize handshake:
    /// 1. Send `initialize` request with client info & capabilities
    /// 2. Validate response protocol version and server info
    /// 3. Send `notifications/initialized` confirmation notification
    async fn initialize(mut self) -> Result<Self> {
        let init_params = json!({
            "protocolVersion": DEFAULT_MCP_PROTOCOL_VERSION,
            "capabilities": {
                "tools": {}
            },
            "clientInfo": {
                "name": "tagisan",
                "version": env!("CARGO_PKG_VERSION")
            }
        });

        let id = self.next_id();
        let req = JsonRpcRequest::new(id, "initialize", Some(init_params));

        let resp = self
            .transport
            .send_request(req, self.default_timeout)
            .await?;

        if let Some(err) = resp.error {
            return Err(TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("MCP server '{}' rejected initialization: {err}", self.server_name),
            ));
        }

        let result_val = resp.result.ok_or_else(|| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                "Missing result payload in initialize response".to_string(),
            )
        })?;

        let init_result: McpInitializeResult = serde_json::from_value(result_val).map_err(|e| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("Failed to parse McpInitializeResult: {e}"),
            )
        })?;

        info!(
            "Connected to MCP server '{}' (protocol version: '{}', server: '{:?}')",
            self.server_name, init_result.protocol_version, init_result.server_info
        );

        self.protocol_version = init_result.protocol_version;
        self.server_info = init_result.server_info;

        // Send notifications/initialized
        let notif = JsonRpcNotification::new("notifications/initialized", None);
        self.transport.send_notification(notif).await?;

        Ok(self)
    }

    /// Send a standard MCP `ping` probe to verify active connectivity and responsiveness
    pub async fn ping(&self) -> Result<()> {
        let id = self.next_id();
        let req = JsonRpcRequest::new(id, "ping", Some(json!({})));

        let resp = self
            .transport
            .send_request(req, self.default_timeout)
            .await?;

        if let Some(err) = resp.error {
            return Err(TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("ping failed on server '{}': {err}", self.server_name),
            ));
        }

        Ok(())
    }

    /// Retrieve the catalog of tools published by this MCP server via `tools/list`
    pub async fn list_tools(&self) -> Result<Vec<McpToolDefinition>> {
        let id = self.next_id();
        let req = JsonRpcRequest::new(id, "tools/list", Some(json!({})));

        let resp = self
            .transport
            .send_request(req, self.default_timeout)
            .await?;

        if let Some(err) = resp.error {
            return Err(TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("tools/list failed on server '{}': {err}", self.server_name),
            ));
        }

        let result_val = resp.result.ok_or_else(|| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                "Missing result in tools/list response".to_string(),
            )
        })?;

        let list_result: McpToolListResult = serde_json::from_value(result_val).map_err(|e| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("Failed to parse McpToolListResult: {e}"),
            )
        })?;

        Ok(list_result.tools)
    }

    /// Invoke a specific tool on this MCP server via `tools/call`
    pub async fn call_tool(
        &self,
        tool_name: &str,
        arguments: Value,
    ) -> Result<McpToolCallResult> {
        let id = self.next_id();
        let params = json!({
            "name": tool_name,
            "arguments": arguments
        });

        let req = JsonRpcRequest::new(id, "tools/call", Some(params));
        let resp = self
            .transport
            .send_request(req, self.default_timeout)
            .await?;

        if let Some(err) = resp.error {
            return Err(TagisanError::Execution(format!(
                "MCP server '{}' returned error for tool '{}': {err}",
                self.server_name, tool_name
            )));
        }

        let result_val = resp.result.ok_or_else(|| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("Missing result payload for tool '{tool_name}' call"),
            )
        })?;

        let call_result: McpToolCallResult = serde_json::from_value(result_val).map_err(|e| {
            TagisanError::BadResponse(
                format!("mcp_{}", self.server_name),
                format!("Failed to parse McpToolCallResult for tool '{tool_name}': {e}"),
            )
        })?;

        Ok(call_result)
    }

    /// Generate monotonically incrementing request ID
    fn next_id(&self) -> u64 {
        self.next_id.fetch_add(1, Ordering::SeqCst)
    }

    /// Gracefully close transport connection
    pub async fn close(&self) {
        self.transport.close().await;
    }
}
