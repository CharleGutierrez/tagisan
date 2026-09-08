use crate::agent::AutonomousAgent;
use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::tools::ToolRegistry;
use std::fs;
use std::path::Path;
use std::sync::Arc;

/// A specialized agent persona defined according to the ECC specification
#[derive(Debug, Clone, PartialEq)]
pub struct EccAgent {
    pub name: String,
    pub description: String,
    pub tools: Vec<String>,
    pub recommended_model: Option<String>,
    pub system_prompt: String,
}

impl EccAgent {
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        tools: Vec<String>,
        recommended_model: Option<String>,
        system_prompt: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            tools,
            recommended_model,
            system_prompt: system_prompt.into(),
        }
    }

    /// Parse an ECC agent definition from a Markdown string with YAML frontmatter
    pub fn parse(content: &str) -> Result<Self> {
        let trimmed = content.trim_start();
        if !trimmed.starts_with("---") {
            return Err(TagisanError::Execution(
                "Invalid ECC agent format: missing leading '---' frontmatter delimiter".to_string(),
            ));
        }

        // Find the second delimiter
        let rest = &trimmed[3..];
        let end_idx = rest.find("\n---").or_else(|| rest.find("\r\n---")).ok_or_else(|| {
            TagisanError::Execution(
                "Invalid ECC agent format: missing closing '---' frontmatter delimiter".to_string(),
            )
        })?;

        let frontmatter_str = &rest[..end_idx];
        let body_start = end_idx + rest[end_idx..].find("---").unwrap() + 3;
        let body = rest[body_start..].trim().to_string();

        let mut name = String::new();
        let mut description = String::new();
        let mut tools = Vec::new();
        let mut recommended_model = None;

        for line in frontmatter_str.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim().to_lowercase();
                let val = val.trim().trim_matches('"').trim_matches('\'').trim();

                match key.as_str() {
                    "name" => name = val.to_string(),
                    "description" => description = val.to_string(),
                    "model" => {
                        if !val.is_empty() {
                            recommended_model = Some(val.to_string());
                        }
                    }
                    "tools" => {
                        // Support comma-separated: tools: read_file, write_file
                        // or array bracket: tools: [read_file, write_file]
                        let clean_val = val.trim_matches('[').trim_matches(']');
                        for t in clean_val.split(',') {
                            let tool_name = t.trim().to_string();
                            if !tool_name.is_empty() {
                                tools.push(tool_name);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        if name.is_empty() {
            return Err(TagisanError::Execution(
                "Invalid ECC agent format: missing 'name' field in frontmatter".to_string(),
            ));
        }

        Ok(Self {
            name,
            description,
            tools,
            recommended_model,
            system_prompt: body,
        })
    }

    /// Load an ECC agent definition from a file on disk
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path_ref = path.as_ref();
        let content = fs::read_to_string(path_ref).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to read ECC agent file '{}': {}",
                path_ref.display(),
                e
            ))
        })?;
        Self::parse(&content)
    }

    /// Convert this ECC persona into a ready-to-run Tagisan AutonomousAgent
    pub fn into_autonomous_agent(
        &self,
        provider: Arc<dyn LlmProvider>,
        model_override: Option<String>,
        registry: ToolRegistry,
    ) -> AutonomousAgent {
        let model = model_override
            .or_else(|| self.recommended_model.clone())
            .unwrap_or_else(|| "claude-3-5-sonnet-20241022".to_string());

        AutonomousAgent::new(provider, model, registry)
            .with_system_prompt(self.system_prompt.clone())
            .with_agentshield(true)
    }
}
