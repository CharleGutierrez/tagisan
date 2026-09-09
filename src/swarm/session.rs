use crate::error::{Result, TagisanError};
use crate::types::{ContentBlock, Message, Role, TokenUsage};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, info};

/// Full persistent session record capturing agent/swarm conversational state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub model: String,
    pub agent_persona: Option<String>,
    pub system_prompt: Option<String>,
    pub messages: Vec<Message>,
    pub total_usage: TokenUsage,
    pub total_cost_usd: f64,
    pub metadata: HashMap<String, String>,
}

impl SessionRecord {
    pub fn new(id: impl Into<String>, title: impl Into<String>, model: impl Into<String>) -> Self {
        let now = chrono_now_iso();
        Self {
            id: id.into(),
            title: title.into(),
            created_at: now.clone(),
            updated_at: now,
            model: model.into(),
            agent_persona: None,
            system_prompt: None,
            messages: Vec::new(),
            total_usage: TokenUsage::default(),
            total_cost_usd: 0.0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_persona(mut self, persona: impl Into<String>) -> Self {
        self.agent_persona = Some(persona.into());
        self
    }

    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn add_message(&mut self, msg: Message) {
        self.messages.push(msg);
        self.updated_at = chrono_now_iso();
    }

    pub fn to_metadata(&self) -> SessionMetadata {
        SessionMetadata {
            id: self.id.clone(),
            title: self.title.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            model: self.model.clone(),
            agent_persona: self.agent_persona.clone(),
            message_count: self.messages.len(),
            total_cost_usd: self.total_cost_usd,
        }
    }
}

/// Compact session summary for listing stored sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub id: String,
    pub title: String,
    pub created_at: String,
    pub updated_at: String,
    pub model: String,
    pub agent_persona: Option<String>,
    pub message_count: usize,
    pub total_cost_usd: f64,
}

/// Production atomic session persistence store
#[derive(Debug, Clone)]
pub struct SessionStore {
    base_dir: PathBuf,
}

impl Default for SessionStore {
    fn default() -> Self {
        Self::new()
    }
}

impl SessionStore {
    /// Create a session store located in the default `.tagisan/sessions` directory
    pub fn new() -> Self {
        Self {
            base_dir: PathBuf::from(".tagisan").join("sessions"),
        }
    }

    /// Create a session store located in a custom directory
    pub fn with_dir(path: impl Into<PathBuf>) -> Self {
        Self {
            base_dir: path.into(),
        }
    }

    /// Get the base storage path
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    fn ensure_dir(&self) -> Result<()> {
        if !self.base_dir.exists() {
            fs::create_dir_all(&self.base_dir).map_err(|e| {
                TagisanError::Execution(format!(
                    "Failed to create session store directory at {:?}: {e}",
                    self.base_dir
                ))
            })?;
        }
        Ok(())
    }

    fn session_path(&self, id: &str) -> PathBuf {
        let safe_id: String = id
            .chars()
            .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
            .collect();
        self.base_dir.join(format!("{safe_id}.json"))
    }

    /// Atomically persist a session record to disk via temporary file rename
    pub fn save(&self, session: &SessionRecord) -> Result<()> {
        self.ensure_dir()?;
        let target_path = self.session_path(&session.id);
        let temp_path = target_path.with_extension(format!("tmp.{}", std::process::id()));

        let json_bytes = serde_json::to_vec_pretty(session).map_err(|e| {
            TagisanError::Execution(format!("Failed to serialize session '{}': {e}", session.id))
        })?;

        fs::write(&temp_path, &json_bytes).map_err(|e| {
            TagisanError::Execution(format!(
                "Failed to write temp session file at {:?}: {e}",
                temp_path
            ))
        })?;

        fs::rename(&temp_path, &target_path).map_err(|e| {
            // Attempt to clean up temp file if rename fails
            let _ = fs::remove_file(&temp_path);
            TagisanError::Execution(format!(
                "Failed to atomically rename session file to {:?}: {e}",
                target_path
            ))
        })?;

        debug!("Persisted session '{}' to {:?}", session.id, target_path);
        Ok(())
    }

    /// Load a session record by its unique ID
    pub fn load(&self, id: &str) -> Result<SessionRecord> {
        let path = self.session_path(id);
        if !path.exists() {
            return Err(TagisanError::Execution(format!(
                "Session '{id}' not found at {:?}",
                path
            )));
        }

        let bytes = fs::read(&path).map_err(|e| {
            TagisanError::Execution(format!("Failed to read session file at {:?}: {e}", path))
        })?;

        let record: SessionRecord = serde_json::from_slice(&bytes).map_err(|e| {
            TagisanError::Execution(format!("Failed to parse session JSON for '{id}': {e}"))
        })?;

        Ok(record)
    }

    /// List all saved sessions sorted by updated_at descending
    pub fn list(&self) -> Result<Vec<SessionMetadata>> {
        if !self.base_dir.exists() {
            return Ok(Vec::new());
        }

        let entries = fs::read_dir(&self.base_dir).map_err(|e| {
            TagisanError::Execution(format!("Failed to read session dir {:?}: {e}", self.base_dir))
        })?;

        let mut list = Vec::new();
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                if let Ok(bytes) = fs::read(&path) {
                    if let Ok(record) = serde_json::from_slice::<SessionRecord>(&bytes) {
                        list.push(record.to_metadata());
                    }
                }
            }
        }

        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        Ok(list)
    }

    /// Delete a session by its unique ID
    pub fn delete(&self, id: &str) -> Result<bool> {
        let path = self.session_path(id);
        if path.exists() {
            fs::remove_file(&path).map_err(|e| {
                TagisanError::Execution(format!("Failed to delete session file at {:?}: {e}", path))
            })?;
            info!("Deleted session '{}'", id);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Export a full session to a clean GitHub-style Markdown transcript
    pub fn export_markdown(&self, id: &str) -> Result<String> {
        let record = self.load(id)?;
        let mut md = String::new();

        md.push_str(&format!("# Session Transcript: {}\n\n", record.title));
        md.push_str(&format!("- **Session ID**: `{}`\n", record.id));
        md.push_str(&format!("- **Created**: {}\n", record.created_at));
        md.push_str(&format!("- **Updated**: {}\n", record.updated_at));
        md.push_str(&format!("- **Model**: `{}`\n", record.model));
        if let Some(ref persona) = record.agent_persona {
            md.push_str(&format!("- **Persona**: `{}`\n", persona));
        }
        md.push_str(&format!(
            "- **Total Tokens**: {} (prompt: {}, completion: {})\n",
            record.total_usage.prompt_tokens + record.total_usage.completion_tokens,
            record.total_usage.prompt_tokens,
            record.total_usage.completion_tokens
        ));
        md.push_str(&format!("- **Estimated Cost**: ${:.4} USD\n\n", record.total_cost_usd));
        md.push_str("---\n\n");

        for (idx, msg) in record.messages.iter().enumerate() {
            let role_badge = match msg.role {
                Role::User => "👤 **User**",
                Role::Assistant => "🤖 **Assistant**",
                Role::System => "⚙️ **System**",
                Role::Tool => "🛠️ **Tool Results**",
                Role::Reasoning => "🧠 **Reasoning**",
            };

            md.push_str(&format!("### Message #{} — {}\n\n", idx + 1, role_badge));

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        md.push_str(text);
                        md.push_str("\n\n");
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        md.push_str("> 💭 **Internal Reasoning**\n>\n");
                        for line in thinking.lines() {
                            md.push_str(&format!("> {line}\n"));
                        }
                        md.push_str("\n\n");
                    }
                    ContentBlock::ToolCall { id, name, arguments } => {
                        md.push_str(&format!(
                            "**Tool Call** (`{name}`, ID: `{id}`):\n```json\n{}\n```\n\n",
                            serde_json::to_string_pretty(arguments).unwrap_or_else(|_| arguments.to_string())
                        ));
                    }
                    ContentBlock::ToolResult { tool_call_id, content, is_error } => {
                        let status = if *is_error { "🚨 Error" } else { "✅ Success" };
                        md.push_str(&format!(
                            "**Tool Output** (ID: `{tool_call_id}`, Status: {status}):\n```\n{}\n```\n\n",
                            content.trim()
                        ));
                    }
                    ContentBlock::Image { media_type, .. } => {
                        md.push_str(&format!("*[Attached Image: {media_type}]*\n\n"));
                    }
                }
            }
        }

        Ok(md)
    }
}

/// Fallback simple ISO timestamp generator without external chrono dependency
fn chrono_now_iso() -> String {
    use std::time::SystemTime;
    let now = SystemTime::now();
    let dur = now
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Rough ISO formatting
    format!("timestamp-epoch-{}", secs)
}
