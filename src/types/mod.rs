use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Entity producing a message in the conversation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
    /// Dedicated role for explicit reasoning / thinking tokens
    Reasoning,
}

/// Content block within a message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    Image { media_type: String, data_base64: String },
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    ToolResult {
        tool_call_id: String,
        content: String,
        is_error: bool,
    },
}

/// Universal message format across all models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: Vec<ContentBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Message {
    pub fn system(text: impl Into<String>) -> Self {
        Self {
            role: Role::System,
            content: vec![ContentBlock::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn user(text: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn assistant(text: impl Into<String>) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::Text { text: text.into() }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn reasoning(thinking: impl Into<String>) -> Self {
        Self {
            role: Role::Reasoning,
            content: vec![ContentBlock::Thinking {
                thinking: thinking.into(),
                signature: None,
            }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn extract_text(&self) -> String {
        self.content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    pub fn extract_thinking(&self) -> Option<String> {
        let thoughts: Vec<&str> = self
            .content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::Thinking { thinking, .. } => Some(thinking.as_str()),
                _ => None,
            })
            .collect();
        if thoughts.is_empty() {
            None
        } else {
            Some(thoughts.join("\n"))
        }
    }
}

/// Granular incremental chunk emitted during real-time streaming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StreamChunkDelta {
    Text(String),
    Thinking(String),
    ToolCallDelta {
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamChunk {
    pub delta: StreamChunkDelta,
    pub finish_reason: Option<FinishReason>,
    pub usage: Option<TokenUsage>,
}

impl StreamChunk {
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            delta: StreamChunkDelta::Text(text.into()),
            finish_reason: None,
            usage: None,
        }
    }

    pub fn thinking(thinking: impl Into<String>) -> Self {
        Self {
            delta: StreamChunkDelta::Thinking(thinking.into()),
            finish_reason: None,
            usage: None,
        }
    }

    pub fn done(finish_reason: FinishReason, usage: Option<TokenUsage>) -> Self {
        Self {
            delta: StreamChunkDelta::Text(String::new()),
            finish_reason: Some(finish_reason),
            usage,
        }
    }
}

bitflags! {
    /// Capabilities supported by a model
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct ProviderCapabilities: u32 {
        const STREAMING            = 1 << 0;
        const FUNCTION_CALLING     = 1 << 1;
        const VISION               = 1 << 2;
        const SYSTEM_PROMPT        = 1 << 3;
        const REASONING_EXTRACTION = 1 << 4;
        const PROMPT_CACHING       = 1 << 5;
        const JSON_SCHEMA_OUTPUT   = 1 << 6;
    }
}

/// Token usage tracking per call
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub reasoning_tokens: Option<u32>,
    pub cached_prompt_tokens: Option<u32>,
    pub estimated_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ToolCalls,
    ContentFilter,
    Other(String),
}

/// Tool definition schema for function calling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

/// Universal completion request payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionRequest {
    pub model: String,
    pub messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default)]
    pub stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system_prompt: Option<String>,
}

impl CompletionRequest {
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            messages: vec![Message::user(prompt)],
            temperature: Some(0.7),
            max_tokens: Some(4096),
            stream: false,
            system_prompt: None,
        }
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system_prompt = Some(system.into());
        self
    }

    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = stream;
        self
    }

    pub fn with_messages(mut self, messages: Vec<Message>) -> Self {
        self.messages = messages;
        self
    }
}

/// Universal completion response payload
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub message: Message,
    pub finish_reason: FinishReason,
    pub usage: TokenUsage,
    pub latency: std::time::Duration,
}

/// Stateful conversation session manager
#[derive(Debug, Clone, Default)]
pub struct ChatSession {
    pub system_prompt: Option<String>,
    pub history: Vec<Message>,
}

impl ChatSession {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_system(mut self, system: impl Into<String>) -> Self {
        self.system_prompt = Some(system.into());
        self
    }

    pub fn add_user_message(&mut self, text: impl Into<String>) {
        self.history.push(Message::user(text));
    }

    pub fn add_assistant_message(&mut self, text: impl Into<String>) {
        self.history.push(Message::assistant(text));
    }

    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }

    pub fn build_request(&self, model: impl Into<String>) -> CompletionRequest {
        CompletionRequest {
            model: model.into(),
            messages: self.history.clone(),
            temperature: Some(0.7),
            max_tokens: Some(4096),
            stream: false,
            system_prompt: self.system_prompt.clone(),
        }
    }
}
