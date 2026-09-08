use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message, Role, TokenUsage,
};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct AnthropicProvider {
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize)]
struct AnthropicMessagesPayload<'a> {
    model: &'a str,
    messages: Vec<AnthropicMessage<'a>>,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicApiResponse {
    id: String,
    content: Vec<AnthropicContentBlock>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlock {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    cache_creation_input_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = "https://api.anthropic.com/v1/messages";

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| TagisanError::Authentication("anthropic".into(), e.to_string()))?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));

        let mut anthropic_messages = Vec::new();
        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                _ => "user",
            };
            anthropic_messages.push(AnthropicMessage {
                role: role_str,
                content: msg.extract_text(),
            });
        }

        let max_tokens = req.max_tokens.unwrap_or(4096);
        let payload = AnthropicMessagesPayload {
            model: &req.model,
            messages: anthropic_messages,
            max_tokens,
            temperature: req.temperature,
            system: req.system_prompt.as_deref(),
        };

        let response = self
            .client
            .post(url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited("anthropic".into(), None));
            }
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(TagisanError::Authentication("anthropic".into(), err_text));
            }
            return Err(TagisanError::BadResponse("anthropic".into(), err_text));
        }

        let api_resp: AnthropicApiResponse = response.json().await?;
        let mut content_blocks = Vec::new();

        for block in api_resp.content {
            match block {
                AnthropicContentBlock::Text { text } => {
                    content_blocks.push(ContentBlock::Text { text });
                }
                AnthropicContentBlock::Thinking { thinking, signature } => {
                    content_blocks.push(ContentBlock::Thinking { thinking, signature });
                }
                AnthropicContentBlock::ToolUse { id, name, input } => {
                    content_blocks.push(ContentBlock::ToolCall {
                        id,
                        name,
                        arguments: input,
                    });
                }
            }
        }

        let finish_reason = match api_resp.stop_reason.as_deref() {
            Some("end_turn") | Some("stop_sequence") => FinishReason::Stop,
            Some("max_tokens") => FinishReason::Length,
            Some("tool_use") => FinishReason::ToolCalls,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };

        let usage = match api_resp.usage {
            Some(u) => TokenUsage {
                prompt_tokens: u.input_tokens.unwrap_or(0),
                completion_tokens: u.output_tokens.unwrap_or(0),
                reasoning_tokens: None,
                cached_prompt_tokens: u.cache_read_input_tokens,
                estimated_cost_usd: None,
            },
            None => TokenUsage::default(),
        };

        Ok(CompletionResponse {
            id: api_resp.id,
            provider: "anthropic".to_string(),
            model: req.model,
            message: Message {
                role: Role::Assistant,
                content: content_blocks,
                name: None,
                metadata: std::collections::HashMap::new(),
            },
            finish_reason,
            usage,
            latency: start.elapsed(),
        })
    }
}
