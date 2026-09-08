use bitflags::bitflags;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio_util::sync::CancellationToken;

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

impl ContentBlock {
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    pub fn thinking(thinking: impl Into<String>, signature: Option<String>) -> Self {
        Self::Thinking {
            thinking: thinking.into(),
            signature,
        }
    }

    pub fn image(media_type: impl Into<String>, data_base64: impl Into<String>) -> Self {
        Self::Image {
            media_type: media_type.into(),
            data_base64: data_base64.into(),
        }
    }

    pub fn tool_call(id: impl Into<String>, name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self::ToolCall {
            id: id.into(),
            name: name.into(),
            arguments,
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>, is_error: bool) -> Self {
        Self::ToolResult {
            tool_call_id: tool_call_id.into(),
            content: content.into(),
            is_error,
        }
    }
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

    pub fn tool_call(id: impl Into<String>, name: impl Into<String>, arguments: serde_json::Value) -> Self {
        Self {
            role: Role::Assistant,
            content: vec![ContentBlock::ToolCall {
                id: id.into(),
                name: name.into(),
                arguments,
            }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn tool_result(tool_call_id: impl Into<String>, content: impl Into<String>, is_error: bool) -> Self {
        Self {
            role: Role::Tool,
            content: vec![ContentBlock::ToolResult {
                tool_call_id: tool_call_id.into(),
                content: content.into(),
                is_error,
            }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn image(media_type: impl Into<String>, data_base64: impl Into<String>) -> Self {
        Self {
            role: Role::User,
            content: vec![ContentBlock::Image {
                media_type: media_type.into(),
                data_base64: data_base64.into(),
            }],
            name: None,
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    pub fn extract_text(&self) -> String {
        let texts: Vec<String> = self
            .content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::Text { text } => Some(text.clone()),
                ContentBlock::Thinking { thinking, .. } if self.role == Role::Reasoning => Some(thinking.clone()),
                _ => None,
            })
            .collect();
        texts.join("\n")
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

    pub fn extract_tool_calls(&self) -> Vec<(&str, &str, &serde_json::Value)> {
        self.content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::ToolCall { id, name, arguments } => Some((id.as_str(), name.as_str(), arguments)),
                _ => None,
            })
            .collect()
    }

    pub fn extract_tool_results(&self) -> Vec<(&str, &str, bool)> {
        self.content
            .iter()
            .filter_map(|c| match c {
                ContentBlock::ToolResult { tool_call_id, content, is_error } => {
                    Some((tool_call_id.as_str(), content.as_str(), *is_error))
                }
                _ => None,
            })
            .collect()
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

    pub fn tool_call_delta(
        index: usize,
        id: Option<String>,
        name: Option<String>,
        arguments_delta: Option<String>,
    ) -> Self {
        Self {
            delta: StreamChunkDelta::ToolCallDelta {
                index,
                id,
                name,
                arguments_delta,
            },
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

impl ToolDefinition {
    pub fn new(name: impl Into<String>, description: impl Into<String>, parameters: serde_json::Value) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<ToolDefinition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,
    #[serde(skip)]
    pub cancellation_token: Option<CancellationToken>,
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
            tools: Vec::new(),
            tool_choice: None,
            cancellation_token: None,
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

    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
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

    pub fn with_tools(mut self, tools: Vec<ToolDefinition>) -> Self {
        self.tools = tools;
        self
    }

    pub fn with_tool(mut self, tool: ToolDefinition) -> Self {
        self.tools.push(tool);
        self
    }

    pub fn with_tool_choice(mut self, tool_choice: serde_json::Value) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    pub fn with_cancellation(mut self, token: CancellationToken) -> Self {
        self.cancellation_token = Some(token);
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
            tools: Vec::new(),
            tool_choice: None,
            cancellation_token: None,
        }
    }
}
