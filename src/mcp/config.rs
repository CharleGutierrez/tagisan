use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration file representing MCP servers (`mcp.json` / `tagisan.mcp.json`)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct McpConfig {
    #[serde(rename = "mcpServers", default)]
    pub mcp_servers: HashMap<String, McpServerConfig>,
}

/// Configuration for an individual MCP server process
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct McpServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl McpConfig {
    /// Create a new empty MCP configuration
    pub fn new() -> Self {
        Self {
            mcp_servers: HashMap::new(),
        }
    }

    /// Add a server configuration
    pub fn with_server(
        mut self,
        name: impl Into<String>,
        command: impl Into<String>,
        args: Vec<String>,
    ) -> Self {
        self.mcp_servers.insert(
            name.into(),
            McpServerConfig {
                command: command.into(),
                args,
                env: HashMap::new(),
            },
        );
        self
    }

    /// Parse configuration from a JSON string
    pub fn from_json(json_str: &str) -> Result<Self> {
        serde_json::from_str(json_str).map_err(|e| {
            TagisanError::BadResponse(
                "mcp_config".to_string(),
                format!("Failed to parse MCP configuration JSON: {e}"),
            )
        })
    }

    /// Load configuration from a specific file path
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read MCP config file '{}': {e}",
                path_ref.display()
            ))
        })?;
        Self::from_json(&content)
    }

    /// Search for default MCP configuration files in standard locations:
    /// 1. `.tagisan/mcp.json`
    /// 2. `tagisan.mcp.json`
    /// 3. `mcp.json`
    /// 4. `.mcp.json`
    pub fn discover_default() -> Option<(PathBuf, Self)> {
        let candidates = [
            ".tagisan/mcp.json",
            "tagisan.mcp.json",
            "mcp.json",
            ".mcp.json",
        ];

        for candidate in candidates {
            let p = Path::new(candidate);
            if p.is_file() {
                if let Ok(cfg) = Self::from_file(p) {
                    return Some((p.to_path_buf(), cfg));
                }
            }
        }

        None
    }
}
