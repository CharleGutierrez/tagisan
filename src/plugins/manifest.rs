//! Plugin Manifest Specification (`tgs-plugin.toml`)
//!
//! Conforms to RFC-002 for capability-based declarative plugin manifests.

use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Root structure of a `tgs-plugin.toml` manifest
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluginManifest {
    pub plugin: PluginMetadata,
    #[serde(default)]
    pub tools: Option<PluginToolsConfig>,
    #[serde(default)]
    pub skills: Option<PluginSkillsConfig>,
    #[serde(default)]
    pub permissions: PluginPermissions,
    #[serde(default)]
    pub hooks: Option<PluginHooksConfig>,
    #[serde(default)]
    pub mcp: Option<PluginMcpConfig>,
    #[serde(default)]
    pub native: Option<PluginNativeConfig>,
}

impl PluginManifest {
    /// Parse a `tgs-plugin.toml` string
    pub fn parse(content: &str) -> Result<Self> {
        let manifest: PluginManifest = toml::from_str(content).map_err(|e| {
            TagisanError::Execution(format!("Failed to parse tgs-plugin.toml manifest: {e}"))
        })?;
        manifest.validate_format()?;
        Ok(manifest)
    }

    /// Load and parse a `tgs-plugin.toml` file from a directory or direct file path
    pub fn load_from(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let file_path = if path.is_dir() {
            path.join("tgs-plugin.toml")
        } else {
            path.to_path_buf()
        };

        if !file_path.exists() {
            return Err(TagisanError::Execution(format!(
                "Plugin manifest not found at {}",
                file_path.display()
            )));
        }

        let content = std::fs::read_to_string(&file_path).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read plugin manifest {}: {e}",
                file_path.display()
            ))
        })?;

        Self::parse(&content)
    }

    /// Validates core metadata invariants
    pub fn validate_format(&self) -> Result<()> {
        if self.plugin.name.trim().is_empty() {
            return Err(TagisanError::Execution("Plugin 'name' cannot be empty".into()));
        }
        if !self.plugin.name.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err(TagisanError::Execution(format!(
                "Invalid plugin name '{}': must only contain alphanumeric characters, '-', or '_'",
                self.plugin.name
            )));
        }
        Ok(())
    }

    /// Validate relative file path resolutions against the plugin root directory
    pub fn validate_filesystem(&self, plugin_root: &Path) -> Result<()> {
        let entrypoint = self.resolve_entrypoint(plugin_root);
        // Only validate if not MCP with external command or URL
        if self.plugin.runtime == PluginRuntimeType::Mcp {
            if let Some(ref mcp) = self.mcp {
                if mcp.command.is_some() || mcp.sse_url.is_some() {
                    return Ok(());
                }
            }
        }

        if !entrypoint.exists() {
            return Err(TagisanError::Execution(format!(
                "Plugin entrypoint '{}' does not exist in '{}'",
                self.plugin.entrypoint,
                plugin_root.display()
            )));
        }
        Ok(())
    }

    /// Returns the effective entrypoint resolved against the plugin base directory
    pub fn resolve_entrypoint(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(&self.plugin.entrypoint)
    }

    /// List of enabled tools (if specified, or empty)
    pub fn enabled_tool_names(&self) -> Vec<String> {
        self.tools
            .as_ref()
            .map(|t| t.enabled.clone())
            .unwrap_or_default()
    }

    /// Returns the skills catalog directory if declared
    pub fn resolve_skills_dir(&self, base_dir: &Path) -> Option<PathBuf> {
        self.skills.as_ref().and_then(|s| {
            s.catalog_dir.as_ref().map(|rel| base_dir.join(rel))
        })
    }

    /// Returns permissions or default least-privilege permissions
    pub fn permissions(&self) -> &PluginPermissions {
        &self.permissions
    }
}

/// Core metadata describing a plugin
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    /// Target Execution Engine Runtime
    pub runtime: PluginRuntimeType,
    /// Path to execution entrypoint relative to plugin root
    #[serde(default = "default_entrypoint")]
    pub entrypoint: String,
}

fn default_entrypoint() -> String {
    "index.ts".to_string()
}

/// Target execution engine runtime tier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginRuntimeType {
    /// Tier A: WebAssembly linear memory sandbox via WASI/Extism
    Wasm,
    /// Tier B: Fast TS/JS execution via Tagisan's native Bun runtime
    Bun,
    /// Tier C: Model Context Protocol via stdio or SSE transport
    Mcp,
    /// Tier D: High-privilege dynamic native shared library (.so / .dylib / cdylib)
    Native,
}

impl std::fmt::Display for PluginRuntimeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wasm => write!(f, "wasm"),
            Self::Bun => write!(f, "bun"),
            Self::Mcp => write!(f, "mcp"),
            Self::Native => write!(f, "native"),
        }
    }
}

/// Tool registration configuration inside `tgs-plugin.toml`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginToolsConfig {
    #[serde(default)]
    pub enabled: Vec<String>,
    #[serde(default)]
    pub definitions: Vec<PluginToolDefinition>,
}

/// Explicit tool definition in manifest (optional metadata)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginToolDefinition {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub parameters: Option<serde_json::Value>,
}

/// Skill pack configuration inside `tgs-plugin.toml`
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginSkillsConfig {
    /// Relative path to directory containing SKILL.md packages
    pub catalog_dir: Option<String>,
}

/// Strict capability-based permissions enforced by AgentShield
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginPermissions {
    /// Permitted network hostnames or CIDR patterns (e.g. "api.github.com", "10.0.0.*:5432")
    #[serde(default)]
    pub network: Vec<String>,
    /// Permitted filesystem read paths or glob patterns
    #[serde(default)]
    pub fs_read: Vec<String>,
    /// Permitted filesystem write paths or glob patterns
    #[serde(default)]
    pub fs_write: Vec<String>,
    /// Permitted environment variables passed through from host
    #[serde(default)]
    pub env: Vec<String>,
    /// Whether subprocess execution / shell spawning is permitted
    #[serde(default = "default_false")]
    pub subprocesses: bool,
    /// Maximum execution timeout in seconds (default: 30)
    #[serde(default = "default_timeout_secs")]
    pub timeout_secs: u64,
    /// Explicit permission for native cdylib shared library loading
    #[serde(default = "default_false")]
    pub allow_native: bool,
}

fn default_false() -> bool {
    false
}

fn default_timeout_secs() -> u64 {
    30
}

impl Default for PluginPermissions {
    fn default() -> Self {
        Self {
            network: Vec::new(),
            fs_read: Vec::new(),
            fs_write: Vec::new(),
            env: Vec::new(),
            subprocesses: false,
            timeout_secs: default_timeout_secs(),
            allow_native: false,
        }
    }
}

/// Alias for PluginPermissions representing capability sandbox bounds
pub type PluginCapabilities = PluginPermissions;

/// Cognitive lifecycle extension hooks
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginHooksConfig {
    #[serde(default)]
    pub pipeline_stages: Vec<String>,
    #[serde(default)]
    pub debate_judges: Vec<String>,
    #[serde(default)]
    pub shield_interceptors: Vec<String>,
}

/// Optional stdio or SSE transport configuration for Tier C MCP plugins
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginMcpConfig {
    pub command: Option<String>,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: HashMap<String, String>,
    pub sse_url: Option<String>,
}

/// Optional configuration for Tier D native plugins
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginNativeConfig {
    pub init_symbol: Option<String>,
    pub execute_symbol: Option<String>,
    pub required_abi_version: Option<u32>,
}
