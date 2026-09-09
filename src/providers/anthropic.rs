use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
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

    fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            "x-api-key",
            HeaderValue::from_str(&self.api_key)
                .map_err(|e| TagisanError::Authentication("anthropic".into(), e.to_string()))?,
        );
        headers.insert("anthropic-version", HeaderValue::from_static("2023-06-01"));
        headers.insert(
            "anthropic-beta",
            HeaderValue::from_static("prompt-caching-2024-07-31"),
        );
        Ok(headers)
    }

    fn format_messages<'a>(&self, req: &'a CompletionRequest) -> Vec<AnthropicMessage<'a>> {
        let mut anthropic_messages: Vec<AnthropicMessage<'a>> = Vec::new();

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "assistant",
                Role::Tool => "user", // Tool results are sent in user turns in Anthropic API
                Role::Reasoning => "assistant",
                Role::System => "user",
            };

            let mut blocks = Vec::new();
            for content in &msg.content {
                match content {
                    ContentBlock::Text { text } => {
                        blocks.push(AnthropicContentBlockPayload::Text {
                            text: text.as_str(),
                            cache_control: None,
                        });
                    }
                    ContentBlock::Thinking { thinking, signature } => {
                        blocks.push(AnthropicContentBlockPayload::Thinking {
                            thinking: thinking.as_str(),
                            signature: signature.as_deref(),
                        });
                    }
                    ContentBlock::Image { media_type, data_base64 } => {
                        blocks.push(AnthropicContentBlockPayload::Image {
                            source: AnthropicImageSource {
                                source_type: "base64",
                                media_type: media_type.as_str(),
                                data: data_base64.as_str(),
                            },
                        });
                    }
                    ContentBlock::ToolCall { id, name, arguments } => {
                        blocks.push(AnthropicContentBlockPayload::ToolUse {
                            id: id.as_str(),
                            name: name.as_str(),
                            input: arguments,
                        });
                    }
                    ContentBlock::ToolResult { tool_call_id, content, is_error } => {
                        blocks.push(AnthropicContentBlockPayload::ToolResult {
                            tool_use_id: tool_call_id.as_str(),
                            content: content.as_str(),
                            is_error: if *is_error { Some(true) } else { None },
                        });
                    }
                }
            }

            if let Some(last) = anthropic_messages.last_mut() {
                if last.role == role_str {
                    last.content.extend(blocks);
                    continue;
                }
            }

            anthropic_messages.push(AnthropicMessage {
                role: role_str,
                content: blocks,
            });
        }

        anthropic_messages
    }

    fn build_system_prompt(&self, req: &CompletionRequest) -> Option<Vec<AnthropicSystemBlock>> {
        req.system_prompt.as_ref().map(|s| {
            vec![AnthropicSystemBlock {
                block_type: "text",
                text: s.clone(),
                cache_control: Some(AnthropicCacheControl {
                    control_type: "ephemeral".to_string(),
                }),
            }]
        })
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
    system: Option<Vec<AnthropicSystemBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<AnthropicTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<AnthropicThinkingConfig>,
    #[serde(default)]
    stream: bool,
}

#[derive(Serialize)]
struct AnthropicTool<'a> {
    name: &'a str,
    description: &'a str,
    input_schema: &'a serde_json::Value,
}

#[derive(Serialize)]
struct AnthropicThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: &'static str,
    budget_tokens: u32,
}

#[derive(Serialize)]
struct AnthropicSystemBlock {
    #[serde(rename = "type")]
    block_type: &'static str,
    text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    cache_control: Option<AnthropicCacheControl>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct AnthropicCacheControl {
    #[serde(rename = "type")]
    control_type: String,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: Vec<AnthropicContentBlockPayload<'a>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlockPayload<'a> {
    Text {
        text: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        cache_control: Option<AnthropicCacheControl>,
    },
    Thinking {
        thinking: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<&'a str>,
    },
    Image {
        source: AnthropicImageSource<'a>,
    },
    ToolUse {
        id: &'a str,
        name: &'a str,
        input: &'a serde_json::Value,
    },
    ToolResult {
        tool_use_id: &'a str,
        content: &'a str,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
}

#[derive(Serialize)]
struct AnthropicImageSource<'a> {
    #[serde(rename = "type")]
    source_type: &'static str,
    media_type: &'a str,
    data: &'a str,
}

#[derive(Deserialize, Debug)]
struct AnthropicApiResponse {
    id: String,
    content: Vec<AnthropicContentBlockResponse>,
    stop_reason: Option<String>,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicContentBlockResponse {
    Text { text: String },
    Thinking { thinking: String, signature: Option<String> },
    ToolUse { id: String, name: String, input: serde_json::Value },
}

#[derive(Deserialize, Debug, Clone)]
struct AnthropicUsage {
    input_tokens: Option<u32>,
    output_tokens: Option<u32>,
    #[allow(dead_code)]
    cache_creation_input_tokens: Option<u32>,
    cache_read_input_tokens: Option<u32>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(dead_code)]
enum AnthropicStreamEvent {
    MessageStart { message: AnthropicStreamMessageStart },
    ContentBlockStart { index: usize, content_block: AnthropicStreamBlockStart },
    ContentBlockDelta { index: usize, delta: AnthropicDelta },
    ContentBlockStop { index: usize },
    MessageDelta { delta: AnthropicMessageDelta, usage: Option<AnthropicUsage> },
    MessageStop,
    Ping,
    Error { error: AnthropicStreamError },
}

#[derive(Deserialize, Debug)]
struct AnthropicStreamError {
    #[allow(dead_code)]
    #[serde(rename = "type")]
    error_type: Option<String>,
    message: String,
}

#[derive(Deserialize, Debug)]
struct AnthropicStreamMessageStart {
    #[allow(dead_code)]
    id: String,
    #[allow(dead_code)]
    model: String,
    usage: Option<AnthropicUsage>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
enum AnthropicStreamBlockStart {
    Text {
        #[allow(dead_code)]
        text: Option<String>,
    },
    Thinking {
        #[allow(dead_code)]
        thinking: Option<String>,
    },
    ToolUse {
        id: String,
        name: String,
    },
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
enum AnthropicDelta {
    TextDelta { text: String },
    ThinkingDelta { thinking: String },
    InputJsonDelta { partial_json: String },
    #[allow(dead_code)]
    SignatureDelta { signature: String },
}

#[derive(Deserialize, Debug)]
struct AnthropicMessageDelta {
    stop_reason: Option<String>,
}

#[async_trait]
impl LlmProvider for AnthropicProvider {
    fn provider_id(&self) -> &'static str {
        "anthropic"
    }

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        let m = model.to_lowercase();
        let mut caps = ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::FUNCTION_CALLING
            | ProviderCapabilities::VISION
            | ProviderCapabilities::PROMPT_CACHING;

        if m.contains("sonnet") || m.contains("opus") || m.contains("3-7") || m.contains("thinking") {
            caps |= ProviderCapabilities::REASONING_EXTRACTION;
        }
        caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = "https://api.anthropic.com/v1/messages";
        let headers = self.build_headers()?;

        let anthropic_messages = self.format_messages(&req);
        let max_tokens = req.max_tokens.unwrap_or(4096);
        let system = self.build_system_prompt(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| AnthropicTool {
                        name: &t.name,
                        description: &t.description,
                        input_schema: &t.parameters,
                    })
                    .collect(),
            )
        } else {
            None
        };

        let thinking = if (req.model.contains("3-7") || req.model.contains("thinking")) && max_tokens > 2048 {
            Some(AnthropicThinkingConfig {
                thinking_type: "enabled",
                budget_tokens: 2048,
            })
        } else {
            None
        };

        let temperature = if thinking.is_some() { None } else { req.temperature };

        let payload = AnthropicMessagesPayload {
            model: &req.model,
            messages: anthropic_messages,
            max_tokens,
            temperature,
            system,
            tools,
            tool_choice: req.tool_choice.as_ref(),
            thinking,
            stream: false,
        };

        let send_future = self
            .client
            .post(url)
            .headers(headers)
            .json(&payload)
            .send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

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
                AnthropicContentBlockResponse::Text { text } => {
                    content_blocks.push(ContentBlock::Text { text });
                }
                AnthropicContentBlockResponse::Thinking { thinking, signature } => {
                    content_blocks.push(ContentBlock::Thinking { thinking, signature });
                }
                AnthropicContentBlockResponse::ToolUse { id, name, input } => {
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

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        let url = "https://api.anthropic.com/v1/messages";
        let headers = self.build_headers()?;

        let anthropic_messages = self.format_messages(&req);
        let max_tokens = req.max_tokens.unwrap_or(4096);
        let system = self.build_system_prompt(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| AnthropicTool {
                        name: &t.name,
                        description: &t.description,
                        input_schema: &t.parameters,
                    })
                    .collect(),
            )
        } else {
            None
        };

        let thinking = if (req.model.contains("3-7") || req.model.contains("thinking")) && max_tokens > 2048 {
            Some(AnthropicThinkingConfig {
                thinking_type: "enabled",
                budget_tokens: 2048,
            })
        } else {
            None
        };

        let temperature = if thinking.is_some() { None } else { req.temperature };

        let payload = AnthropicMessagesPayload {
            model: &req.model,
            messages: anthropic_messages,
            max_tokens,
            temperature,
            system,
            tools,
            tool_choice: req.tool_choice.as_ref(),
            thinking,
            stream: true,
        };

        let send_future = self
            .client
            .post(url)
            .headers(headers)
            .json(&payload)
            .send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

        let status = response.status();
        if !status.is_success() {
            let err_text = response.text().await.unwrap_or_default();
            if status.as_u16() == 429 {
                return Err(TagisanError::RateLimited("anthropic".into(), None));
            }
            return Err(TagisanError::BadResponse("anthropic".into(), err_text));
        }

        let mut event_stream = response.bytes_stream().eventsource();
        let cancellation_token = req.cancellation_token.clone();

        let output_stream = async_stream::stream! {
            let mut accumulated_prompt_tokens = 0u32;
            let mut accumulated_output_tokens = 0u32;
            let mut accumulated_cache_read_tokens = None;
            let mut active_tools: std::collections::HashMap<usize, (String, String)> = std::collections::HashMap::new();
            let mut last_finish_reason: Option<FinishReason> = None;

            loop {
                let next_event = if let Some(ref token) = cancellation_token {
                    tokio::select! {
                        _ = token.cancelled() => {
                            yield Err(TagisanError::Cancelled);
                            return;
                        }
                        evt = event_stream.next() => evt,
                    }
                } else {
                    event_stream.next().await
                };

                let event_res = match next_event {
                    Some(res) => res,
                    None => break,
                };

                match event_res {
                    Ok(event) => {
                        let data = event.data.trim();
                        if data.is_empty() {
                            continue;
                        }
                        match serde_json::from_str::<AnthropicStreamEvent>(data) {
                            Ok(AnthropicStreamEvent::MessageStart { message }) => {
                                if let Some(u) = message.usage {
                                    accumulated_prompt_tokens = u.input_tokens.unwrap_or(0);
                                    accumulated_cache_read_tokens = u.cache_read_input_tokens;
                                }
                            }
                            Ok(AnthropicStreamEvent::ContentBlockStart {
                                index,
                                content_block: AnthropicStreamBlockStart::ToolUse { id, name },
                            }) => {
                                active_tools.insert(index, (id.clone(), name.clone()));
                                yield Ok(StreamChunk {
                                    delta: StreamChunkDelta::ToolCallDelta {
                                        index,
                                        id: Some(id),
                                        name: Some(name),
                                        arguments_delta: None,
                                    },
                                    finish_reason: None,
                                    usage: None,
                                });
                            }
                            Ok(AnthropicStreamEvent::ContentBlockDelta { index, delta }) => {
                                match delta {
                                    AnthropicDelta::TextDelta { text } => {
                                        yield Ok(StreamChunk::text(text));
                                    }
                                    AnthropicDelta::ThinkingDelta { thinking } => {
                                        yield Ok(StreamChunk::thinking(thinking));
                                    }
                                    AnthropicDelta::InputJsonDelta { partial_json } => {
                                        let (id, name) = active_tools.get(&index).cloned().unwrap_or_default();
                                        yield Ok(StreamChunk {
                                            delta: StreamChunkDelta::ToolCallDelta {
                                                index,
                                                id: if id.is_empty() { None } else { Some(id) },
                                                name: if name.is_empty() { None } else { Some(name) },
                                                arguments_delta: Some(partial_json),
                                            },
                                            finish_reason: None,
                                            usage: None,
                                        });
                                    }
                                    AnthropicDelta::SignatureDelta { .. } => {}
                                }
                            }
                            Ok(AnthropicStreamEvent::ContentBlockStop { .. }) => {}
                            Ok(AnthropicStreamEvent::MessageDelta { delta, usage }) => {
                                let reason = match delta.stop_reason.as_deref() {
                                    Some("tool_use") => Some(FinishReason::ToolCalls),
                                    Some("max_tokens") => Some(FinishReason::Length),
                                    Some("end_turn") | Some("stop_sequence") => Some(FinishReason::Stop),
                                    Some(other) => Some(FinishReason::Other(other.to_string())),
                                    None => None,
                                };
                                if reason.is_some() {
                                    last_finish_reason = reason.clone();
                                }
                                let out_tokens = usage.as_ref().and_then(|u| u.output_tokens).unwrap_or(0);
                                accumulated_output_tokens = out_tokens;
                                let tok_usage = TokenUsage {
                                    prompt_tokens: accumulated_prompt_tokens,
                                    completion_tokens: out_tokens,
                                    reasoning_tokens: None,
                                    cached_prompt_tokens: accumulated_cache_read_tokens,
                                    estimated_cost_usd: None,
                                };
                                yield Ok(StreamChunk {
                                    delta: StreamChunkDelta::Text(String::new()),
                                    finish_reason: reason,
                                    usage: Some(tok_usage),
                                });
                            }
                            Ok(AnthropicStreamEvent::MessageStop) => {
                                let final_usage = Some(TokenUsage {
                                    prompt_tokens: accumulated_prompt_tokens,
                                    completion_tokens: accumulated_output_tokens,
                                    reasoning_tokens: None,
                                    cached_prompt_tokens: accumulated_cache_read_tokens,
                                    estimated_cost_usd: None,
                                });
                                let final_reason = last_finish_reason.clone().unwrap_or(FinishReason::Stop);
                                yield Ok(StreamChunk::done(final_reason, final_usage));
                            }
                            Ok(AnthropicStreamEvent::Error { error }) => {
                                yield Err(TagisanError::BadResponse("anthropic".into(), error.message));
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        yield Err(TagisanError::BadResponse("anthropic".into(), e.to_string()));
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}
