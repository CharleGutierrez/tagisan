use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration file representing MCP servers (`mcp.json` / `tagisan.mcp.json` / `mcp.dynamic.json`)
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authority: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subsystem: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auto_approve: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl McpServerConfig {
    /// Create a new server configuration with command and arguments
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            env: HashMap::new(),
            authority: None,
            subsystem: None,
            auto_approve: None,
            description: None,
        }
    }

    /// Expand environment variables (e.g. `${ENV_VAR}` or `${ENV_VAR:-default}`) across command, args, and env map
    pub fn expand_env(&self) -> Self {
        let mut expanded = self.clone();
        expanded.expand_in_place();
        expanded
    }

    /// Expand environment variables in-place across command, args, and env map
    pub fn expand_in_place(&mut self) {
        self.command = expand_env_vars(&self.command);
        for arg in &mut self.args {
            *arg = expand_env_vars(arg);
        }
        let mut expanded_env = HashMap::with_capacity(self.env.len());
        for (k, v) in &self.env {
            expanded_env.insert(expand_env_vars(k), expand_env_vars(v));
        }
        self.env = expanded_env;
    }
}

/// Expand environment variables in format `${VAR}` or `${VAR:-default}` in a string
pub fn expand_env_vars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '$' && chars.peek() == Some(&'{') {
            chars.next(); // consume '{'
            let mut var_expr = String::new();
            let mut closed = false;
            for inner_c in chars.by_ref() {
                if inner_c == '}' {
                    closed = true;
                    break;
                }
                var_expr.push(inner_c);
            }

            if closed {
                if let Some((var_name, default_val)) = var_expr.split_once(":-") {
                    let val = std::env::var(var_name.trim())
                        .ok()
                        .filter(|v| !v.is_empty())
                        .unwrap_or_else(|| default_val.to_string());
                    result.push_str(&val);
                } else {
                    let val = std::env::var(var_expr.trim()).unwrap_or_default();
                    result.push_str(&val);
                }
            } else {
                // Not closed, preserve literal
                result.push('$');
                result.push('{');
                result.push_str(&var_expr);
            }
        } else {
            result.push(c);
        }
    }

    result
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
                ..Default::default()
            },
        );
        self
    }

    /// Add a server configuration to this config
    pub fn add_server(&mut self, name: impl Into<String>, config: McpServerConfig) {
        self.mcp_servers.insert(name.into(), config);
    }

    /// Remove a server configuration by name, returning the removed configuration if found
    pub fn remove_server(&mut self, name: &str) -> Option<McpServerConfig> {
        self.mcp_servers.remove(name)
    }

    /// Expand environment variables across all configured MCP servers
    pub fn expand_env(&self) -> Self {
        let mut expanded = self.clone();
        expanded.expand_in_place();
        expanded
    }

    /// Expand environment variables in-place across all configured MCP servers
    pub fn expand_in_place(&mut self) {
        for server in self.mcp_servers.values_mut() {
            server.expand_in_place();
        }
    }

    /// Save configuration to a specific file path formatted as pretty JSON
    pub fn save_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        let path_ref = path.as_ref();
        if let Some(parent) = path_ref.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    TagisanError::Execution(format!(
                        "Failed to create parent directory '{}': {e}",
                        parent.display()
                    ))
                })?;
            }
        }
        let serialized = serde_json::to_string_pretty(self).map_err(TagisanError::Serialization)?;
        fs::write(path_ref, serialized).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to write MCP config file '{}': {e}",
                path_ref.display()
            ))
        })?;
        Ok(())
    }

    /// Save configuration to dynamic MCP configuration file (`mcp.dynamic.json`)
    pub fn save_dynamic(&self) -> Result<PathBuf> {
        let path = PathBuf::from("mcp.dynamic.json");
        self.save_to_file(&path)?;
        Ok(path)
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
    /// 3. `mcp.dynamic.json`
    /// 4. `mcp.json`
    /// 5. `.mcp.json`
    pub fn discover_default() -> Option<(PathBuf, Self)> {
        let candidates = [
            ".tagisan/mcp.json",
            "tagisan.mcp.json",
            "mcp.dynamic.json",
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
