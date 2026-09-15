use crate::ecc::{AgentShieldScanner, AgentShieldVerdict};
use crate::error::{Result, TagisanError};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Inspectable status of active IDE integrations within a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeIntegrationStatus {
    pub vscode_configured: bool,
    pub cursor_configured: bool,
    pub windsurf_configured: bool,
    pub claude_configured: bool,
    pub zed_configured: bool,
    pub jetbrains_configured: bool,
    pub detected_editors: Vec<String>,
}

/// Resolves the absolute path to the active `tgs` executable
pub fn resolve_tgs_executable() -> String {
    if let Ok(exe) = std::env::current_exe() {
        if let Ok(canonical) = exe.canonicalize() {
            return canonical.to_string_lossy().to_string();
        }
        return exe.to_string_lossy().to_string();
    }
    "tgs".to_string()
}

/// Safely writes content to a file after strict AgentShield cyber defense validation
fn safe_write_file(path: &Path, content: &str) -> Result<PathBuf> {
    let path_str = path.to_string_lossy();
    let verdict = AgentShieldScanner::scan_file_path(&path_str);
    if let AgentShieldVerdict::Block { reason, threat_level } = verdict {
        return Err(TagisanError::Security(format!(
            "AgentShield blocked file operation on {:?} [{:?}]: {}",
            path, threat_level, reason
        )));
    }

    if let Some(parent) = path.parent() {
        let parent_str = parent.to_string_lossy();
        let parent_verdict = AgentShieldScanner::scan_file_path(&parent_str);
        if let AgentShieldVerdict::Block { reason, threat_level } = parent_verdict {
            return Err(TagisanError::Security(format!(
                "AgentShield blocked directory creation for {:?} [{:?}]: {}",
                parent, threat_level, reason
            )));
        }
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(path, content)?;
    Ok(path.to_path_buf())
}

/// Generator for production-ready IDE configuration bundles
#[derive(Debug, Clone)]
pub struct IdeConfigGenerator {
    pub base_path: PathBuf,
    pub exe_path: String,
}

impl IdeConfigGenerator {
    /// Create a new generator targeting the given base directory
    pub fn new(base_path: impl Into<PathBuf>) -> Self {
        Self {
            base_path: base_path.into(),
            exe_path: resolve_tgs_executable(),
        }
    }

    /// Override the executable path used in generated configs
    pub fn with_exe_path(mut self, exe_path: impl Into<String>) -> Self {
        self.exe_path = exe_path.into();
        self
    }

    /// Generate VS Code configuration files (`.vscode/settings.json`, `.vscode/tasks.json`, `.vscode/extensions.json`)
    pub fn generate_vscode(&self) -> Result<Vec<PathBuf>> {
        let mut generated = Vec::new();
        let vscode_dir = self.base_path.join(".vscode");

        // 1. settings.json
        let settings_file = vscode_dir.join("settings.json");
        let settings_json = json!({
            "tagisan.enable": true,
            "tagisan.lsp.serverPath": self.exe_path,
            "tagisan.lsp.args": ["ide", "lsp"],
            "tagisan.mcp.enabled": true,
            "tagisan.agentshield.enabled": true
        });
        generated.push(safe_write_file(
            &settings_file,
            &serde_json::to_string_pretty(&settings_json)?,
        )?);

        // 2. tasks.json (Tagisan Debate, Agent, Verify, ECC tasks)
        let tasks_file = vscode_dir.join("tasks.json");
        let tasks_json = json!({
            "version": "2.0.0",
            "tasks": [
                {
                    "label": "Tagisan: Dialectical Debate",
                    "type": "shell",
                    "command": self.exe_path,
                    "args": ["debate", "${input:debatePrompt}"],
                    "problemMatcher": [],
                    "group": "build"
                },
                {
                    "label": "Tagisan: Autonomous Agent",
                    "type": "shell",
                    "command": self.exe_path,
                    "args": ["agent", "${input:agentTask}"],
                    "problemMatcher": [],
                    "group": "build"
                },
                {
                    "label": "Tagisan: Verify Invariants (Ground)",
                    "type": "shell",
                    "command": self.exe_path,
                    "args": ["ground", "${file}"],
                    "problemMatcher": [],
                    "group": "test"
                },
                {
                    "label": "Tagisan: ECC Multi-Agent Pipeline",
                    "type": "shell",
                    "command": self.exe_path,
                    "args": ["ecc", "pipeline", "${input:eccObjective}"],
                    "problemMatcher": [],
                    "group": "build"
                }
            ],
            "inputs": [
                {
                    "id": "debatePrompt",
                    "description": "Enter the architecture or technical question to debate",
                    "default": "Analyze system architecture and invariants",
                    "type": "promptString"
                },
                {
                    "id": "agentTask",
                    "description": "Enter the goal for the autonomous agent",
                    "default": "Implement feature and verify test suite",
                    "type": "promptString"
                },
                {
                    "id": "eccObjective",
                    "description": "Enter the objective for the ECC pipeline",
                    "default": "Implement, test, review and verify module",
                    "type": "promptString"
                }
            ]
        });
        generated.push(safe_write_file(
            &tasks_file,
            &serde_json::to_string_pretty(&tasks_json)?,
        )?);

        // 3. extensions.json
        let extensions_file = vscode_dir.join("extensions.json");
        let extensions_json = json!({
            "recommendations": [
                "rust-lang.rust-analyzer"
            ]
        });
        generated.push(safe_write_file(
            &extensions_file,
            &serde_json::to_string_pretty(&extensions_json)?,
        )?);

        Ok(generated)
    }

    /// Generate Cursor configuration files (`.cursor/mcp.json`, `.cursorrules`)
    pub fn generate_cursor(&self) -> Result<Vec<PathBuf>> {
        let mut generated = Vec::new();
        let cursor_dir = self.base_path.join(".cursor");

        // .cursor/mcp.json
        let mcp_file = cursor_dir.join("mcp.json");
        let mut mcp_val = if mcp_file.exists() {
            std::fs::read_to_string(&mcp_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({})
        };

        if !mcp_val.is_object() {
            mcp_val = json!({});
        }
        if mcp_val.get("mcpServers").is_none() {
            mcp_val["mcpServers"] = json!({});
        }
        mcp_val["mcpServers"]["tagisan"] = json!({
            "command": self.exe_path,
            "args": ["mcp", "serve"]
        });

        generated.push(safe_write_file(
            &mcp_file,
            &serde_json::to_string_pretty(&mcp_val)?,
        )?);

        // .cursorrules
        let cursorrules_file = self.base_path.join(".cursorrules");
        let cursorrules_content = r#"# Tagisan AI Development Guidelines & Skills Integration

When assisting with this codebase:
1. Prioritize Tagisan dialectical consensus (`tgs debate`) for major architectural decisions.
2. Utilize Tagisan MCP tools: `view_file`, `replace_file_content`, `grep_search`, `find_by_name`.
3. Ensure all proposed file modifications adhere to AgentShield cyber defense guardrails.
4. Leverage the 100 Agentic and 50 Architecture skills embedded within Tagisan.
5. Before applying critical refactors, verify AST invariants with `tgs ground`.
"#;
        generated.push(safe_write_file(&cursorrules_file, cursorrules_content)?);

        Ok(generated)
    }

    /// Generate Windsurf configuration files (`.codeium/windsurf/mcp_config.json`)
    pub fn generate_windsurf(&self) -> Result<Vec<PathBuf>> {
        let windsurf_dir = self.base_path.join(".codeium").join("windsurf");
        let mcp_file = windsurf_dir.join("mcp_config.json");

        let mut mcp_val = if mcp_file.exists() {
            std::fs::read_to_string(&mcp_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({})
        };

        if !mcp_val.is_object() {
            mcp_val = json!({});
        }
        if mcp_val.get("mcpServers").is_none() {
            mcp_val["mcpServers"] = json!({});
        }
        mcp_val["mcpServers"]["tagisan"] = json!({
            "command": self.exe_path,
            "args": ["mcp", "serve"]
        });

        let written = safe_write_file(&mcp_file, &serde_json::to_string_pretty(&mcp_val)?)?;
        Ok(vec![written])
    }

    /// Generate Claude Desktop configuration entry (`claude_desktop_config.json`)
    pub fn generate_claude(&self) -> Result<Vec<PathBuf>> {
        let config_file = self.base_path.join("claude_desktop_config.json");
        let mut config_val = if config_file.exists() {
            std::fs::read_to_string(&config_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({})
        };

        if !config_val.is_object() {
            config_val = json!({});
        }
        if config_val.get("mcpServers").is_none() {
            config_val["mcpServers"] = json!({});
        }
        config_val["mcpServers"]["tagisan"] = json!({
            "command": self.exe_path,
            "args": ["mcp", "serve"]
        });

        let written = safe_write_file(&config_file, &serde_json::to_string_pretty(&config_val)?)?;
        Ok(vec![written])
    }

    /// Generate Zed configuration files (`.zed/settings.json`)
    pub fn generate_zed(&self) -> Result<Vec<PathBuf>> {
        let zed_dir = self.base_path.join(".zed");
        let settings_file = zed_dir.join("settings.json");

        let mut settings_val = if settings_file.exists() {
            std::fs::read_to_string(&settings_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({})
        };

        if !settings_val.is_object() {
            settings_val = json!({});
        }
        if settings_val.get("context_servers").is_none() {
            settings_val["context_servers"] = json!({});
        }
        settings_val["context_servers"]["tagisan"] = json!({
            "command": self.exe_path,
            "args": ["mcp", "serve"]
        });

        let written = safe_write_file(
            &settings_file,
            &serde_json::to_string_pretty(&settings_val)?,
        )?;
        Ok(vec![written])
    }

    /// Generate JetBrains MCP server configuration (`tagisan.mcp.json`)
    pub fn generate_jetbrains(&self) -> Result<Vec<PathBuf>> {
        let jetbrains_file = self.base_path.join("tagisan.mcp.json");
        let mut mcp_val = if jetbrains_file.exists() {
            std::fs::read_to_string(&jetbrains_file)
                .ok()
                .and_then(|s| serde_json::from_str::<Value>(&s).ok())
                .unwrap_or_else(|| json!({}))
        } else {
            json!({})
        };

        if !mcp_val.is_object() {
            mcp_val = json!({});
        }
        if mcp_val.get("mcpServers").is_none() {
            mcp_val["mcpServers"] = json!({});
        }
        mcp_val["mcpServers"]["tagisan"] = json!({
            "command": self.exe_path,
            "args": ["mcp", "serve"]
        });

        let written = safe_write_file(
            &jetbrains_file,
            &serde_json::to_string_pretty(&mcp_val)?,
        )?;
        Ok(vec![written])
    }

    /// Generate configuration for a given target (`all`, `vscode`, `cursor`, `windsurf`, `claude`, `zed`, `jetbrains`)
    pub fn generate_target(&self, target: &str) -> Result<Vec<PathBuf>> {
        match target.to_lowercase().as_str() {
            "all" => self.generate_all(),
            "vscode" => self.generate_vscode(),
            "cursor" => self.generate_cursor(),
            "windsurf" => self.generate_windsurf(),
            "claude" | "claude_desktop" => self.generate_claude(),
            "zed" => self.generate_zed(),
            "jetbrains" => self.generate_jetbrains(),
            other => Err(TagisanError::Execution(format!(
                "Unsupported IDE target '{}'. Supported: all, vscode, cursor, windsurf, claude, zed, jetbrains",
                other
            ))),
        }
    }

    /// Generate all IDE configuration bundles simultaneously
    pub fn generate_all(&self) -> Result<Vec<PathBuf>> {
        let mut all_files = Vec::new();
        all_files.extend(self.generate_vscode()?);
        all_files.extend(self.generate_cursor()?);
        all_files.extend(self.generate_windsurf()?);
        all_files.extend(self.generate_claude()?);
        all_files.extend(self.generate_zed()?);
        all_files.extend(self.generate_jetbrains()?);
        Ok(all_files)
    }
}

/// Inspect active IDE configurations and detect integrations in a directory
pub fn inspect_ide_status(base_path: impl AsRef<Path>) -> IdeIntegrationStatus {
    let path = base_path.as_ref();
    let vscode = path.join(".vscode/settings.json").exists() && path.join(".vscode/tasks.json").exists();
    let cursor = path.join(".cursor/mcp.json").exists();
    let windsurf = path.join(".codeium/windsurf/mcp_config.json").exists();
    let claude = path.join("claude_desktop_config.json").exists();
    let zed = path.join(".zed/settings.json").exists();
    let jetbrains = path.join("tagisan.mcp.json").exists();

    let mut detected_editors = Vec::new();
    if vscode {
        detected_editors.push("VS Code".to_string());
    }
    if cursor {
        detected_editors.push("Cursor".to_string());
    }
    if windsurf {
        detected_editors.push("Windsurf".to_string());
    }
    if claude {
        detected_editors.push("Claude Desktop".to_string());
    }
    if zed {
        detected_editors.push("Zed".to_string());
    }
    if jetbrains {
        detected_editors.push("JetBrains".to_string());
    }

    IdeIntegrationStatus {
        vscode_configured: vscode,
        cursor_configured: cursor,
        windsurf_configured: windsurf,
        claude_configured: claude,
        zed_configured: zed,
        jetbrains_configured: jetbrains,
        detected_editors,
    }
}
