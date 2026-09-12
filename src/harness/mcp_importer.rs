use colored::Colorize;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{Result, TagisanError};
use crate::harness::generator::{GeneratedHarness, HarnessGenerator};
use crate::harness::skill_packager::SkillPackager;
use crate::harness::spec::{ArgumentSpec, ArgumentType, CommandSpec, HarnessSpec, SourceType};

/// Ingestion result for an imported MCP tool manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpImportResult {
    pub harness_name: String,
    pub tools_imported: usize,
    pub output_dir: PathBuf,
    pub cli_path: PathBuf,
    pub test_path: PathBuf,
    pub skill_path: Option<PathBuf>,
}

impl McpImportResult {
    pub fn display_summary(&self) -> String {
        let mut out = String::new();
        out.push_str(&format!(
            "{} Transpiled MCP manifest into agent-native harness '{}'\n",
            "✔ [MCP IMPORTED]".green().bold(),
            self.harness_name.cyan().bold()
        ));
        out.push_str(&format!("  • Tools Imported: {} commands\n", self.tools_imported));
        out.push_str(&format!("  • Standalone CLI: {}\n", self.cli_path.display().to_string().green()));
        out.push_str(&format!("  • Test Harness:   {}\n", self.test_path.display().to_string().green()));
        if let Some(ref sk) = self.skill_path {
            out.push_str(&format!("  • Installed Skill: {}\n", sk.display().to_string().cyan().bold()));
        }
        out
    }
}

/// Bi-directional MCP (Model Context Protocol) Schema Importer & Transpiler
pub struct McpImporter;

impl McpImporter {
    /// Transpiles an MCP tool JSON schema file into an agent-native CLI harness and RFC-004 SKILL.md
    pub async fn import(
        spec_source: &str,
        custom_name: Option<String>,
        output_dir: Option<PathBuf>,
        install: bool,
    ) -> Result<McpImportResult> {
        let (raw_json, source_path) = if Path::new(spec_source).is_file() {
            let p = PathBuf::from(spec_source);
            let s = fs::read_to_string(&p).map_err(|e| {
                TagisanError::Execution(format!("Failed to read MCP spec file '{}': {}", p.display(), e))
            })?;
            (s, p)
        } else {
            (spec_source.to_string(), PathBuf::from("mcp_manifest.json"))
        };

        let parsed: Value = serde_json::from_str(&raw_json).map_err(|e| {
            TagisanError::Execution(format!("Invalid JSON in MCP specification: {}", e))
        })?;

        let harness_name = custom_name.unwrap_or_else(|| {
            if let Some(n) = parsed.get("name").and_then(|v| v.as_str()) {
                n.to_string()
            } else if let Some(stem) = source_path.file_stem().and_then(|s| s.to_str()) {
                stem.to_string()
            } else {
                "mcp_tools".to_string()
            }
        });

        let spec = Self::convert_mcp_to_spec(&harness_name, &source_path, &parsed)?;
        let tools_imported = spec.commands.len();

        let generated = HarnessGenerator::generate(&spec)?;

        let target_dir = output_dir.unwrap_or_else(|| {
            Path::new(".tagisan")
                .join("harness")
                .join(&harness_name.to_lowercase().replace('_', "-"))
        });
        fs::create_dir_all(&target_dir).map_err(|e| {
            TagisanError::Execution(format!("Failed to create output dir '{}': {}", target_dir.display(), e))
        })?;

        let (cli_path, test_path) = generated.write_to_dir(&target_dir)?;

        let packaged = SkillPackager::package(&spec, &generated)?;
        let skill_md_path = target_dir.join("SKILL.md");
        fs::write(&skill_md_path, &packaged.skill_md_content).map_err(|e| {
            TagisanError::Execution(format!("Failed to write SKILL.md: {}", e))
        })?;

        let installed_path = if install {
            let skills_root = Path::new(".ecc").join("skills");
            Some(packaged.install_to(&skills_root)?)
        } else {
            None
        };

        Ok(McpImportResult {
            harness_name,
            tools_imported,
            output_dir: target_dir,
            cli_path,
            test_path,
            skill_path: installed_path,
        })
    }

    /// Converts parsed MCP JSON structure into a HarnessSpec
    pub fn convert_mcp_to_spec(name: &str, source_path: &Path, val: &Value) -> Result<HarnessSpec> {
        let mut spec = HarnessSpec::new(name, source_path, SourceType::Auto);
        spec.description = format!("Agent-native CLI harness transpiled from MCP specification {}", name);

        // 1. Identify tools list
        let tools_list: Vec<&Value> = if let Some(tools) = val.get("tools").and_then(|t| t.as_array()) {
            tools.iter().collect()
        } else if let Some(arr) = val.as_array() {
            arr.iter().collect()
        } else if val.get("inputSchema").is_some() || val.get("name").is_some() {
            vec![val]
        } else if let Some(servers) = val.get("mcpServers").and_then(|s| s.as_object()) {
            let mut list = Vec::new();
            for (_srv_name, srv_val) in servers {
                if let Some(tools) = srv_val.get("tools").and_then(|t| t.as_array()) {
                    list.extend(tools.iter());
                }
            }
            list
        } else {
            Vec::new()
        };

        if tools_list.is_empty() {
            return Err(TagisanError::Execution(
                "No valid MCP tool definitions found in specification. Expected 'tools' array or tool schema.".to_string(),
            ));
        }

        for tool in tools_list {
            let tool_name = tool
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("unnamed_tool")
                .to_string();
            let norm_cmd_name = tool_name.replace('_', "-").to_lowercase();

            let tool_desc = tool
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("")
                .to_string();

            let mut cmd = CommandSpec::new(&norm_cmd_name, &tool_name).with_description(&tool_desc);

            // Parse inputSchema
            if let Some(schema) = tool.get("inputSchema") {
                let required_props: Vec<String> = schema
                    .get("required")
                    .and_then(|r| r.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                if let Some(properties) = schema.get("properties").and_then(|p| p.as_object()) {
                    for (prop_name, prop_val) in properties {
                        let arg_type = Self::json_schema_type_to_arg_type(prop_name, prop_val);
                        let prop_desc = prop_val
                            .get("description")
                            .and_then(|d| d.as_str())
                            .unwrap_or("")
                            .to_string();

                        let is_required = required_props.contains(prop_name);
                        let default_val = prop_val.get("default").map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        });

                        let mut arg = ArgumentSpec::new(prop_name, arg_type)
                            .with_description(prop_desc)
                            .with_required(is_required);

                        if let Some(def) = default_val {
                            arg = arg.with_default(def);
                        }

                        cmd = cmd.with_argument(arg);
                    }
                }
            }

            spec = spec.with_command(cmd);
        }

        Ok(spec)
    }

    /// Maps JSON Schema type annotations to typed ArgumentType
    fn json_schema_type_to_arg_type(prop_name: &str, prop_val: &Value) -> ArgumentType {
        if let Some(enums) = prop_val.get("enum").and_then(|e| e.as_array()) {
            let choices: Vec<String> = enums
                .iter()
                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                .collect();
            if !choices.is_empty() {
                return ArgumentType::Choice(choices);
            }
        }

        let type_str = prop_val
            .get("type")
            .and_then(|t| t.as_str())
            .unwrap_or("string")
            .to_lowercase();

        match type_str.as_str() {
            "integer" => ArgumentType::Integer,
            "number" => ArgumentType::Float,
            "boolean" => ArgumentType::Boolean,
            "array" => {
                let inner_type = prop_val
                    .get("items")
                    .and_then(|it| it.get("type"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("string");
                let inner = match inner_type {
                    "integer" => ArgumentType::Integer,
                    "number" => ArgumentType::Float,
                    _ => ArgumentType::String,
                };
                ArgumentType::List(Box::new(inner))
            }
            "object" => ArgumentType::Dict,
            _ => {
                let name_lower = prop_name.to_lowercase();
                if name_lower.contains("path") || name_lower.contains("file") || name_lower.contains("dir") {
                    ArgumentType::FilePath
                } else {
                    ArgumentType::String
                }
            }
        }
    }
}
