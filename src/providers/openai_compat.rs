use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use std::time::Instant;

pub struct OpenAiCompatibleProvider {
    provider_id: &'static str,
    base_url: String,
    api_key: String,
    client: reqwest::Client,
}

impl OpenAiCompatibleProvider {
    pub fn new(provider_id: &'static str, base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            provider_id,
            base_url: base_url.into(),
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new("openai", "https://api.openai.com/v1", api_key)
    }

    pub fn xai(api_key: impl Into<String>) -> Self {
        Self::new("xai", "https://api.x.ai/v1", api_key)
    }

    pub fn deepseek(api_key: impl Into<String>) -> Self {
        Self::new("deepseek", "https://api.deepseek.com", api_key)
    }

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| TagisanError::Authentication(self.provider_id.to_string(), e.to_string()))?,
        );
        Ok(headers)
    }
}

#[derive(Serialize)]
struct ChatCompletionPayload<'a> {
    model: &'a str,
    messages: Vec<ApiMessage<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(default)]
    stream: bool,
}

#[derive(Serialize)]
struct ApiMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionApiResponse {
    id: Option<String>,
    choices: Vec<ApiChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct ApiChoice {
    message: ApiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct ApiResponseMessage {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize)]
struct StreamChatCompletionChunk {
    choices: Vec<StreamChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize)]
struct StreamChoice {
    delta: StreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct StreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
}

#[derive(Deserialize, Clone)]
struct ApiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_id(&self) -> &'static str {
        self.provider_id
    }

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        let m = model.to_lowercase();
        let mut caps = ProviderCapabilities::STREAMING | ProviderCapabilities::SYSTEM_PROMPT | ProviderCapabilities::FUNCTION_CALLING;

        if m.contains("r1") || m.contains("reasoner") || m.contains("o1") || m.contains("o3") {
            caps |= ProviderCapabilities::REASONING_EXTRACTION;
        }
        if m.contains("deepseek") {
            caps |= ProviderCapabilities::PROMPT_CACHING;
        }
        if m.contains("gpt-4o") || m.contains("grok-vision") {
            caps |= ProviderCapabilities::VISION;
        }
        caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let headers = self.build_headers()?;

        let mut api_messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            api_messages.push(ApiMessage {
                role: "system",
                content: sys.clone(),
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "tool",
                Role::Reasoning => "assistant",
            };
            api_messages.push(ApiMessage {
                role: role_str,
                content: msg.extract_text(),
            });
        }

        let payload = ChatCompletionPayload {
            model: &req.model,
            messages: api_messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: false,
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited(self.provider_id.to_string(), None));
            }
            if status.as_u16() == 401 || status.as_u16() == 403 {
                return Err(TagisanError::Authentication(self.provider_id.to_string(), err_text));
            }
            return Err(TagisanError::BadResponse(self.provider_id.to_string(), err_text));
        }

        let api_resp: ChatCompletionApiResponse = response.json().await?;
        let choice = api_resp
            .choices
            .into_iter()
            .next()
            .ok_or_else(|| TagisanError::BadResponse(self.provider_id.to_string(), "No choices in response".into()))?;

        let mut content_blocks = Vec::new();

        if let Some(reasoning) = choice.message.reasoning_content {
            if !reasoning.trim().is_empty() {
                content_blocks.push(ContentBlock::Thinking {
                    thinking: reasoning,
                    signature: None,
                });
            }
        }

        let raw_text = choice.message.content.unwrap_or_default();

        if raw_text.contains("<think>") && raw_text.contains("</think>") {
            if let Some(start_idx) = raw_text.find("<think>") {
                if let Some(end_idx) = raw_text.find("</think>") {
                    let thinking_str = &raw_text[start_idx + 7..end_idx].trim();
                    content_blocks.push(ContentBlock::Thinking {
                        thinking: thinking_str.to_string(),
                        signature: None,
                    });
                    let clean_text = format!("{}{}", &raw_text[..start_idx], &raw_text[end_idx + 8..]).trim().to_string();
                    content_blocks.push(ContentBlock::Text { text: clean_text });
                }
            }
        } else if !raw_text.is_empty() {
            content_blocks.push(ContentBlock::Text { text: raw_text });
        }

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };

        let usage = match api_resp.usage {
            Some(u) => TokenUsage {
                prompt_tokens: u.prompt_tokens.unwrap_or(0),
                completion_tokens: u.completion_tokens.unwrap_or(0),
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: None,
            },
            None => TokenUsage::default(),
        };

        Ok(CompletionResponse {
            id: api_resp.id.unwrap_or_else(|| "id_gen".to_string()),
            provider: self.provider_id.to_string(),
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

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let headers = self.build_headers()?;

        let mut api_messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            api_messages.push(ApiMessage {
                role: "system",
                content: sys.clone(),
            });
        }

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::System => "system",
                Role::User => "user",
                Role::Assistant => "assistant",
                _ => "user",
            };
            api_messages.push(ApiMessage {
                role: role_str,
                content: msg.extract_text(),
            });
        }

        let payload = ChatCompletionPayload {
            model: &req.model,
            messages: api_messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: true,
        };

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited(self.provider_id.to_string(), None));
            }
            return Err(TagisanError::BadResponse(self.provider_id.to_string(), err_text));
        }

        let provider_id = self.provider_id;
        let event_stream = response.bytes_stream().eventsource();

        let mapped = event_stream.filter_map(move |event_res| {
            async move {
                match event_res {
                    Ok(event) => {
                        let data = event.data.trim();
                        if data == "[DONE]" {
                            return Some(Ok(StreamChunk::done(FinishReason::Stop, None)));
                        }
                        if data.is_empty() {
                            return None;
                        }

                        match serde_json::from_str::<StreamChatCompletionChunk>(data) {
                            Ok(chunk) => {
                                let usage = chunk.usage.map(|u| TokenUsage {
                                    prompt_tokens: u.prompt_tokens.unwrap_or(0),
                                    completion_tokens: u.completion_tokens.unwrap_or(0),
                                    reasoning_tokens: None,
                                    cached_prompt_tokens: None,
                                    estimated_cost_usd: None,
                                });

                                if let Some(choice) = chunk.choices.into_iter().next() {
                                    if let Some(reasoning) = choice.delta.reasoning_content {
                                        if !reasoning.is_empty() {
                                            return Some(Ok(StreamChunk {
                                                delta: StreamChunkDelta::Thinking(reasoning),
                                                finish_reason: None,
                                                usage,
                                            }));
                                        }
                                    }
                                    if let Some(content) = choice.delta.content {
                                        if !content.is_empty() {
                                            return Some(Ok(StreamChunk {
                                                delta: StreamChunkDelta::Text(content),
                                                finish_reason: choice.finish_reason.map(|_| FinishReason::Stop),
                                                usage,
                                            }));
                                        }
                                    }
                                }
                                None
                            }
                            Err(e) => {
                                tracing::debug!("Failed to parse stream chunk: {}", e);
                                None
                            }
                        }
                    }
                    Err(e) => Some(Err(TagisanError::BadResponse(provider_id.to_string(), e.to_string()))),
                }
            }
        });

        Ok(Box::pin(mapped))
    }
}
