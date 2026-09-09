use crate::error::{Result, TagisanError};
use crate::mcp::adapter::McpToolWrapper;
use crate::mcp::client::McpClient;
use crate::mcp::config::McpConfig;
use crate::mcp::protocol::McpToolDefinition;
use crate::tools::ToolRegistry;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tracing::{info, warn};

/// High-level orchestrator managing multiple MCP server connections and tool registration
pub struct McpManager {
    config: McpConfig,
    clients: HashMap<String, Arc<McpClient>>,
    discovered_tools: HashMap<String, Vec<McpToolDefinition>>,
}

impl McpManager {
    /// Create a new McpManager with the given configuration
    pub fn new(config: McpConfig) -> Self {
        Self {
            config,
            clients: HashMap::new(),
            discovered_tools: HashMap::new(),
        }
    }

    /// Load MCP configuration from a specific file path or standard discovery locations
    pub fn load(path: Option<&Path>) -> Result<Self> {
        let config = if let Some(p) = path {
            McpConfig::from_file(p)?
        } else if let Some((found_path, cfg)) = McpConfig::discover_default() {
            info!("Discovered MCP configuration at '{}'", found_path.display());
            cfg
        } else {
            McpConfig::new()
        };

        Ok(Self::new(config))
    }

    /// Access the underlying configuration
    pub fn config(&self) -> &McpConfig {
        &self.config
    }

    /// List all configured server names
    pub fn server_names(&self) -> Vec<String> {
        self.config.mcp_servers.keys().cloned().collect()
    }

    /// Check if any servers are configured
    pub fn is_empty(&self) -> bool {
        self.config.mcp_servers.is_empty()
    }

    /// Number of configured servers
    pub fn len(&self) -> usize {
        self.config.mcp_servers.len()
    }

    /// Connect to a specific server by name if not already connected
    pub async fn connect_server(&mut self, name: &str) -> Result<Arc<McpClient>> {
        if let Some(client) = self.clients.get(name) {
            return Ok(client.clone());
        }

        let srv_config = self.config.mcp_servers.get(name).ok_or_else(|| {
            TagisanError::Execution(format!(
                "MCP server '{name}' is not configured in mcp.json"
            ))
        })?;

        let client = Arc::new(McpClient::connect(name, srv_config).await?);
        let tools = client.list_tools().await?;

        info!(
            "Discovered {} tool(s) from MCP server '{}'",
            tools.len(),
            name
        );

        self.discovered_tools.insert(name.to_string(), tools);
        self.clients.insert(name.to_string(), client.clone());

        Ok(client)
    }

    /// Connect to all configured MCP servers and discover their published tools
    pub async fn connect_all(&mut self) -> Result<Vec<(String, Vec<McpToolDefinition>)>> {
        let server_names = self.server_names();
        let mut results = Vec::new();

        for name in server_names {
            match self.connect_server(&name).await {
                Ok(_client) => {
                    let tools = self
                        .discovered_tools
                        .get(&name)
                        .cloned()
                        .unwrap_or_default();
                    results.push((name, tools));
                }
                Err(err) => {
                    warn!("Failed to connect to MCP server '{}': {}", name, err);
                }
            }
        }

        Ok(results)
    }

    /// Register all discovered tools across all active MCP servers into a ToolRegistry
    pub fn populate_tool_registry(&self, registry: &mut ToolRegistry, prefix_server: bool) -> usize {
        let mut count = 0;
        for (server_name, client) in &self.clients {
            if let Some(tools) = self.discovered_tools.get(server_name) {
                for def in tools {
                    let wrapper = McpToolWrapper::new(
                        client.clone(),
                        def.clone(),
                        prefix_server,
                    );
                    registry.register_tool(wrapper);
                    count += 1;
                }
            }
        }
        count
    }

    /// Get reference to a connected client
    pub fn get_client(&self, name: &str) -> Option<Arc<McpClient>> {
        self.clients.get(name).cloned()
    }

    /// Get tools discovered for a server
    pub fn get_discovered_tools(&self, name: &str) -> Option<&Vec<McpToolDefinition>> {
        self.discovered_tools.get(name)
    }

    /// Close all active server connections
    pub async fn shutdown_all(&mut self) {
        for (_name, client) in self.clients.drain() {
            client.close().await;
        }
        self.discovered_tools.clear();
    }
}
