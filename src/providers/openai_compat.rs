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
    pub fn new(
        provider_id: &'static str,
        base_url: impl Into<String>,
        api_key: impl Into<String>,
    ) -> Self {
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

    fn format_messages(&self, req: &CompletionRequest) -> Vec<OpenAiMessagePayload> {
        let mut api_messages = Vec::new();

        if let Some(sys) = &req.system_prompt {
            api_messages.push(OpenAiMessagePayload {
                role: "system".to_string(),
                content: Some(serde_json::Value::String(sys.clone())),
                tool_calls: None,
                tool_call_id: None,
                name: None,
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

            let mut tool_calls = Vec::new();
            let mut tool_call_id = None;
            let mut content_parts = Vec::new();
            let mut has_images = false;

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        content_parts.push(serde_json::json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        content_parts.push(serde_json::json!({
                            "type": "text",
                            "text": format!("<think>\n{}\n</think>", thinking)
                        }));
                    }
                    ContentBlock::Image { media_type, data_base64 } => {
                        has_images = true;
                        content_parts.push(serde_json::json!({
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:{};base64,{}", media_type, data_base64)
                            }
                        }));
                    }
                    ContentBlock::ToolCall { id, name, arguments } => {
                        tool_calls.push(OpenAiToolCallSerialization {
                            id: id.clone(),
                            call_type: "function".to_string(),
                            function: OpenAiFunctionCallSerialization {
                                name: name.clone(),
                                arguments: if arguments.is_string() {
                                    arguments.as_str().unwrap().to_string()
                                } else {
                                    arguments.to_string()
                                },
                            },
                        });
                    }
                    ContentBlock::ToolResult { tool_call_id: tid, content, .. } => {
                        tool_call_id = Some(tid.clone());
                        content_parts.push(serde_json::json!({
                            "type": "text",
                            "text": content
                        }));
                    }
                }
            }

            let content_val = if has_images {
                Some(serde_json::Value::Array(content_parts))
            } else if !content_parts.is_empty() {
                let text_combined: Vec<String> = content_parts
                    .iter()
                    .filter_map(|p| p.get("text").and_then(|t| t.as_str()).map(|s| s.to_string()))
                    .collect();
                Some(serde_json::Value::String(text_combined.join("\n")))
            } else {
                None
            };

            api_messages.push(OpenAiMessagePayload {
                role: role_str.to_string(),
                content: content_val,
                tool_calls: if tool_calls.is_empty() { None } else { Some(tool_calls) },
                tool_call_id,
                name: msg.name.clone(),
            });
        }

        api_messages
    }
}

#[derive(Serialize, Debug)]
struct ChatCompletionPayload<'a> {
    model: &'a str,
    messages: Vec<OpenAiMessagePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(default)]
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    stream_options: Option<StreamOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OpenAiTool<'a>>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_choice: Option<&'a serde_json::Value>,
}

#[derive(Serialize, Debug)]
struct StreamOptions {
    include_usage: bool,
}

#[derive(Serialize, Debug)]
struct OpenAiTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: OpenAiFunctionDefinition<'a>,
}

#[derive(Serialize, Debug)]
struct OpenAiFunctionDefinition<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize, Debug)]
struct OpenAiMessagePayload {
    role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_calls: Option<Vec<OpenAiToolCallSerialization>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tool_call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAiToolCallSerialization {
    id: String,
    #[serde(rename = "type")]
    call_type: String,
    function: OpenAiFunctionCallSerialization,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct OpenAiFunctionCallSerialization {
    name: String,
    arguments: String,
}

#[derive(Deserialize, Debug)]
struct ChatCompletionApiResponse {
    id: Option<String>,
    choices: Vec<ApiChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize, Debug)]
struct ApiChoice {
    message: ApiResponseMessage,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ApiResponseMessage {
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<OpenAiToolCallSerialization>>,
}

#[derive(Deserialize, Debug)]
struct StreamChatCompletionChunk {
    choices: Vec<StreamChoice>,
    usage: Option<ApiUsage>,
}

#[derive(Deserialize, Debug)]
struct StreamChoice {
    delta: StreamDelta,
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct StreamDelta {
    content: Option<String>,
    reasoning_content: Option<String>,
    tool_calls: Option<Vec<StreamToolCallDelta>>,
}

#[derive(Deserialize, Debug)]
struct StreamToolCallDelta {
    index: usize,
    id: Option<String>,
    #[allow(dead_code)]
    #[serde(rename = "type")]
    call_type: Option<String>,
    function: Option<StreamFunctionDelta>,
}

#[derive(Deserialize, Debug)]
struct StreamFunctionDelta {
    name: Option<String>,
    arguments: Option<String>,
}

#[derive(Deserialize, Clone, Debug)]
struct ApiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ThinkState {
    Initial,
    InThinking,
    AfterThinking,
}

/// Robust state machine parser to separate real-time `<think>` deltas from standard text deltas
pub struct StreamingThinkParser {
    state: ThinkState,
    buffer: String,
}

impl StreamingThinkParser {
    pub fn new() -> Self {
        Self {
            state: ThinkState::Initial,
            buffer: String::new(),
        }
    }

    pub fn process(&mut self, chunk: &str) -> Vec<StreamChunkDelta> {
        self.buffer.push_str(chunk);
        let mut deltas = Vec::new();

        loop {
            match self.state {
                ThinkState::Initial => {
                    let trimmed = self.buffer.trim_start();
                    if trimmed.starts_with("<think>") {
                        if let Some(pos) = self.buffer.find("<think>") {
                            let before = &self.buffer[..pos];
                            if !before.is_empty() {
                                deltas.push(StreamChunkDelta::Text(before.to_string()));
                            }
                            let remainder = self.buffer[pos + 7..].to_string();
                            self.buffer = remainder;
                            self.state = ThinkState::InThinking;
                            continue;
                        }
                    } else if "<think>".starts_with(trimmed) && !trimmed.is_empty() {
                        // Partial match of opening tag, wait for more chunks
                        break;
                    } else {
                        if let Some(pos) = self.buffer.find("<think>") {
                            let before = &self.buffer[..pos];
                            if !before.is_empty() {
                                deltas.push(StreamChunkDelta::Text(before.to_string()));
                            }
                            let remainder = self.buffer[pos + 7..].to_string();
                            self.buffer = remainder;
                            self.state = ThinkState::InThinking;
                            continue;
                        }
                        if !self.buffer.is_empty() {
                            deltas.push(StreamChunkDelta::Text(std::mem::take(&mut self.buffer)));
                        }
                        self.state = ThinkState::AfterThinking;
                        break;
                    }
                    break;
                }
                ThinkState::InThinking => {
                    if let Some(pos) = self.buffer.find("</think>") {
                        let thinking_text = &self.buffer[..pos];
                        if !thinking_text.is_empty() {
                            deltas.push(StreamChunkDelta::Thinking(thinking_text.to_string()));
                        }
                        let after = self.buffer[pos + 8..].to_string();
                        self.buffer = after;
                        self.state = ThinkState::AfterThinking;
                        continue;
                    } else {
                        let tag = "</think>";
                        let mut partial_len = 0;
                        for i in (1..tag.len()).rev() {
                            if self.buffer.ends_with(&tag[..i]) {
                                partial_len = i;
                                break;
                            }
                        }
                        if partial_len > 0 {
                            let safe_len = self.buffer.len() - partial_len;
                            if safe_len > 0 {
                                let thinking_text = self.buffer[..safe_len].to_string();
                                deltas.push(StreamChunkDelta::Thinking(thinking_text));
                                self.buffer = self.buffer[safe_len..].to_string();
                            }
                        } else if !self.buffer.is_empty() {
                            deltas.push(StreamChunkDelta::Thinking(std::mem::take(&mut self.buffer)));
                        }
                        break;
                    }
                }
                ThinkState::AfterThinking => {
                    if !self.buffer.is_empty() {
                        deltas.push(StreamChunkDelta::Text(std::mem::take(&mut self.buffer)));
                    }
                    break;
                }
            }
        }

        deltas
    }

    pub fn finish(&mut self) -> Vec<StreamChunkDelta> {
        let mut deltas = Vec::new();
        if !self.buffer.is_empty() {
            match self.state {
                ThinkState::InThinking => {
                    deltas.push(StreamChunkDelta::Thinking(std::mem::take(&mut self.buffer)));
                }
                _ => {
                    deltas.push(StreamChunkDelta::Text(std::mem::take(&mut self.buffer)));
                }
            }
        }
        deltas
    }
}

impl Default for StreamingThinkParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_id(&self) -> &'static str {
        self.provider_id
    }

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        let m = model.to_lowercase();
        let mut caps = ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::FUNCTION_CALLING;

        if m.contains("r1") || m.contains("reasoner") || m.contains("o1") || m.contains("o3") {
            caps |= ProviderCapabilities::REASONING_EXTRACTION;
        }
        if m.contains("deepseek") {
            caps |= ProviderCapabilities::PROMPT_CACHING;
        }
        if m.contains("gpt-4o") || m.contains("grok-vision") || m.contains("vision") {
            caps |= ProviderCapabilities::VISION;
        }
        caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));
        let headers = self.build_headers()?;

        let api_messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OpenAiTool {
                        tool_type: "function",
                        function: OpenAiFunctionDefinition {
                            name: &t.name,
                            description: &t.description,
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        } else {
            None
        };

        let payload = ChatCompletionPayload {
            model: &req.model,
            messages: api_messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: false,
            stream_options: None,
            tools,
            tool_choice: req.tool_choice.as_ref(),
        };

        let send_future = self
            .client
            .post(&url)
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

        if let Some(tool_calls) = choice.message.tool_calls {
            for tc in tool_calls {
                let parsed_args = serde_json::from_str::<serde_json::Value>(&tc.function.arguments)
                    .unwrap_or(serde_json::Value::String(tc.function.arguments));
                content_blocks.push(ContentBlock::ToolCall {
                    id: tc.id,
                    name: tc.function.name,
                    arguments: parsed_args,
                });
            }
        }

        let raw_text = choice.message.content.unwrap_or_default();

        if raw_text.contains("<think>") && raw_text.contains("</think>") {
            if let Some(start_idx) = raw_text.find("<think>") {
                if let Some(end_idx) = raw_text.find("</think>") {
                    let thinking_str = raw_text[start_idx + 7..end_idx].trim().to_string();
                    content_blocks.push(ContentBlock::Thinking {
                        thinking: thinking_str,
                        signature: None,
                    });
                    let clean_text = format!("{}{}", &raw_text[..start_idx], &raw_text[end_idx + 8..]).trim().to_string();
                    if !clean_text.is_empty() {
                        content_blocks.push(ContentBlock::Text { text: clean_text });
                    }
                }
            }
        } else if !raw_text.is_empty() {
            content_blocks.push(ContentBlock::Text { text: raw_text });
        }

        let finish_reason = match choice.finish_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some("tool_calls") => FinishReason::ToolCalls,
            Some("content_filter") => FinishReason::ContentFilter,
            Some(other) => FinishReason::Other(other.to_string()),
            None => {
                if content_blocks.iter().any(|b| matches!(b, ContentBlock::ToolCall { .. })) {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
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

        let api_messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OpenAiTool {
                        tool_type: "function",
                        function: OpenAiFunctionDefinition {
                            name: &t.name,
                            description: &t.description,
                            parameters: &t.parameters,
                        },
                    })
                    .collect(),
            )
        } else {
            None
        };

        let payload = ChatCompletionPayload {
            model: &req.model,
            messages: api_messages,
            temperature: req.temperature,
            max_tokens: req.max_tokens,
            stream: true,
            stream_options: Some(StreamOptions { include_usage: true }),
            tools,
            tool_choice: req.tool_choice.as_ref(),
        };

        let send_future = self
            .client
            .post(&url)
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
                return Err(TagisanError::RateLimited(self.provider_id.to_string(), None));
            }
            return Err(TagisanError::BadResponse(self.provider_id.to_string(), err_text));
        }

        let provider_id = self.provider_id;
        let mut event_stream = response.bytes_stream().eventsource();
        let cancellation_token = req.cancellation_token.clone();

        let output_stream = async_stream::stream! {
            let mut think_parser = StreamingThinkParser::new();
            let mut accumulated_usage: Option<TokenUsage> = None;
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
                        if data == "[DONE]" {
                            for delta in think_parser.finish() {
                                yield Ok(StreamChunk {
                                    delta,
                                    finish_reason: None,
                                    usage: None,
                                });
                            }
                            let final_reason = last_finish_reason.unwrap_or(FinishReason::Stop);
                            yield Ok(StreamChunk::done(final_reason, accumulated_usage.clone()));
                            break;
                        }
                        if data.is_empty() {
                            continue;
                        }

                        match serde_json::from_str::<StreamChatCompletionChunk>(data) {
                            Ok(chunk) => {
                                if let Some(u) = chunk.usage {
                                    accumulated_usage = Some(TokenUsage {
                                        prompt_tokens: u.prompt_tokens.unwrap_or(0),
                                        completion_tokens: u.completion_tokens.unwrap_or(0),
                                        reasoning_tokens: None,
                                        cached_prompt_tokens: None,
                                        estimated_cost_usd: None,
                                    });
                                }

                                if let Some(choice) = chunk.choices.into_iter().next() {
                                    if let Some(ref fr) = choice.finish_reason {
                                        last_finish_reason = Some(match fr.as_str() {
                                            "tool_calls" => FinishReason::ToolCalls,
                                            "length" => FinishReason::Length,
                                            "content_filter" => FinishReason::ContentFilter,
                                            _ => FinishReason::Stop,
                                        });
                                    }

                                    if let Some(reasoning) = choice.delta.reasoning_content {
                                        if !reasoning.is_empty() {
                                            yield Ok(StreamChunk {
                                                delta: StreamChunkDelta::Thinking(reasoning),
                                                finish_reason: None,
                                                usage: accumulated_usage.clone(),
                                            });
                                        }
                                    }

                                    if let Some(tool_calls) = choice.delta.tool_calls {
                                        for tc in tool_calls {
                                            yield Ok(StreamChunk {
                                                delta: StreamChunkDelta::ToolCallDelta {
                                                    index: tc.index,
                                                    id: tc.id,
                                                    name: tc.function.as_ref().and_then(|f| f.name.clone()),
                                                    arguments_delta: tc.function.and_then(|f| f.arguments),
                                                },
                                                finish_reason: last_finish_reason.clone(),
                                                usage: accumulated_usage.clone(),
                                            });
                                        }
                                    }

                                    if let Some(content) = choice.delta.content {
                                        if !content.is_empty() {
                                            let deltas = think_parser.process(&content);
                                            for delta in deltas {
                                                yield Ok(StreamChunk {
                                                    delta,
                                                    finish_reason: last_finish_reason.clone(),
                                                    usage: accumulated_usage.clone(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Failed to parse stream chunk: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(TagisanError::BadResponse(provider_id.to_string(), e.to_string()));
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}
