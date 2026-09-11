use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tokio::io::AsyncBufReadExt;
use tokio_util::io::StreamReader;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        let timeout_secs = std::env::var("OLLAMA_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(600);

        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            base_url: base_url.into(),
            client,
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:11434")
    }

    fn format_messages(&self, req: &CompletionRequest) -> Vec<OllamaMessage> {
        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(OllamaMessage {
                role: "system".to_string(),
                content: sys.clone(),
                images: None,
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

            let mut text_parts = Vec::new();
            let mut images = Vec::new();

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        text_parts.push(text.clone());
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        text_parts.push(format!("<think>\n{}\n</think>", thinking));
                    }
                    ContentBlock::Image { data_base64, .. } => {
                        images.push(data_base64.clone());
                    }
                    ContentBlock::ToolCall { name, arguments, .. } => {
                        text_parts.push(format!("[Tool Call: {}({})]", name, arguments));
                    }
                    ContentBlock::ToolResult { tool_call_id, content, .. } => {
                        text_parts.push(format!("[Tool Result for {}: {}]", tool_call_id, content));
                    }
                }
            }

            messages.push(OllamaMessage {
                role: role_str.to_string(),
                content: text_parts.join("\n"),
                images: if images.is_empty() { None } else { Some(images) },
            });
        }

        messages
    }

    fn build_options(&self, req: &CompletionRequest) -> OllamaOptions {
        let num_gpu = std::env::var("TAGISAN_OLLAMA_NUM_GPU")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(Some(99)); // Default to 99: offload all transformer layers into GPU VRAM

        let num_batch = std::env::var("TAGISAN_OLLAMA_NUM_BATCH")
            .ok()
            .and_then(|s| s.parse().ok())
            .or(Some(512)); // High-throughput prompt ingestion batch size

        let num_ctx = std::env::var("TAGISAN_OLLAMA_NUM_CTX")
            .ok()
            .and_then(|s| s.parse().ok())
            .or_else(|| req.max_tokens.map(|m| m.max(4096)));

        let num_thread = std::env::var("TAGISAN_OLLAMA_NUM_THREAD")
            .ok()
            .and_then(|s| s.parse().ok());

        OllamaOptions {
            temperature: req.temperature,
            num_predict: req.max_tokens,
            num_ctx,
            num_gpu,
            num_thread,
            num_batch,
            f16_kv: Some(true),
            use_mmap: Some(true),
            use_mlock: None,
        }
    }

    fn get_keep_alive(&self) -> String {
        std::env::var("TAGISAN_OLLAMA_KEEP_ALIVE").unwrap_or_else(|_| "24h".to_string())
    }
}

/// Advanced inference options passed directly to the llama.cpp engine inside Ollama
#[derive(Serialize, Debug, Clone, Default)]
pub struct OllamaOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_predict: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_ctx: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_gpu: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_thread: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub num_batch: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub f16_kv: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mmap: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub use_mlock: Option<bool>,
}

#[derive(Serialize)]
struct OllamaChatPayload<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<OllamaTool<'a>>>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    keep_alive: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    options: Option<OllamaOptions>,
}

#[derive(Serialize)]
struct OllamaTool<'a> {
    #[serde(rename = "type")]
    tool_type: &'static str,
    function: OllamaFunctionDefinition<'a>,
}

#[derive(Serialize)]
struct OllamaFunctionDefinition<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
struct OllamaApiResponse {
    message: OllamaMessageResponse,
    done: Option<bool>,
    done_reason: Option<String>,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Deserialize, Debug)]
struct OllamaMessageResponse {
    content: String,
    tool_calls: Option<Vec<OllamaToolCallResponse>>,
}

#[derive(Deserialize, Debug)]
struct OllamaToolCallResponse {
    function: OllamaFunctionResponse,
}

#[derive(Deserialize, Debug)]
struct OllamaFunctionResponse {
    name: String,
    arguments: serde_json::Value,
}

#[async_trait]
impl LlmProvider for OllamaProvider {
    fn provider_id(&self) -> &'static str {
        "ollama"
    }

    fn capabilities(&self, _model: &str) -> ProviderCapabilities {
        ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::FUNCTION_CALLING
            | ProviderCapabilities::VISION
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OllamaTool {
                        tool_type: "function",
                        function: OllamaFunctionDefinition {
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

        let keep_alive = self.get_keep_alive();
        let options = self.build_options(&req);

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            tools,
            stream: false,
            keep_alive: Some(&keep_alive),
            options: Some(options),
        };

        let send_future = self.client.post(&url).json(&payload).send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let api_resp: OllamaApiResponse = response.json().await?;

        let mut content_blocks = Vec::new();
        let mut has_tool_calls = false;

        if let Some(tool_calls) = api_resp.message.tool_calls {
            for (idx, tc) in tool_calls.into_iter().enumerate() {
                has_tool_calls = true;
                content_blocks.push(ContentBlock::ToolCall {
                    id: format!("ollama_call_{}", idx),
                    name: tc.function.name,
                    arguments: tc.function.arguments,
                });
            }
        }

        let raw_text = api_resp.message.content;
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

        let finish_reason = match api_resp.done_reason.as_deref() {
            Some("stop") => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
            Some("length") => FinishReason::Length,
            Some(other) => FinishReason::Other(other.to_string()),
            None => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
        };

        let usage = TokenUsage {
            prompt_tokens: api_resp.prompt_eval_count.unwrap_or(0),
            completion_tokens: api_resp.eval_count.unwrap_or(0),
            reasoning_tokens: None,
            cached_prompt_tokens: None,
            estimated_cost_usd: Some(0.0),
        };

        Ok(CompletionResponse {
            id: format!("ollama_{}", start.elapsed().as_millis()),
            provider: "ollama".to_string(),
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
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let messages = self.format_messages(&req);

        let tools = if !req.tools.is_empty() {
            Some(
                req.tools
                    .iter()
                    .map(|t| OllamaTool {
                        tool_type: "function",
                        function: OllamaFunctionDefinition {
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

        let keep_alive = self.get_keep_alive();
        let options = self.build_options(&req);

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            tools,
            stream: true,
            keep_alive: Some(&keep_alive),
            options: Some(options),
        };

        let send_future = self.client.post(&url).json(&payload).send();

        let response = if let Some(token) = &req.cancellation_token {
            tokio::select! {
                _ = token.cancelled() => return Err(TagisanError::Cancelled),
                res = send_future => res?,
            }
        } else {
            send_future.await?
        };

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let cancellation_token = req.cancellation_token.clone();
        let stream = response.bytes_stream().map(|item| {
            item.map_err(std::io::Error::other)
        });
        let reader = StreamReader::new(stream);
        let mut lines = reader.lines();

        let output_stream = async_stream::stream! {
            loop {
                let next_line = if let Some(ref token) = cancellation_token {
                    tokio::select! {
                        _ = token.cancelled() => {
                            yield Err(TagisanError::Cancelled);
                            return;
                        }
                        line_res = lines.next_line() => line_res,
                    }
                } else {
                    lines.next_line().await
                };

                let line_str = match next_line {
                    Ok(Some(line)) => line,
                    Ok(None) => break,
                    Err(e) => {
                        yield Err(TagisanError::BadResponse("ollama".into(), e.to_string()));
                        break;
                    }
                };

                let line_trim = line_str.trim();
                if line_trim.is_empty() {
                    continue;
                }

                match serde_json::from_str::<OllamaApiResponse>(line_trim) {
                    Ok(resp) => {
                        let is_done = resp.done.unwrap_or(false);
                        let usage = if is_done {
                            Some(TokenUsage {
                                prompt_tokens: resp.prompt_eval_count.unwrap_or(0),
                                completion_tokens: resp.eval_count.unwrap_or(0),
                                reasoning_tokens: None,
                                cached_prompt_tokens: None,
                                estimated_cost_usd: Some(0.0),
                            })
                        } else {
                            None
                        };

                        if let Some(tool_calls) = resp.message.tool_calls {
                            for (idx, tc) in tool_calls.into_iter().enumerate() {
                                yield Ok(StreamChunk {
                                    delta: StreamChunkDelta::ToolCallDelta {
                                        index: idx,
                                        id: Some(format!("ollama_call_{}", idx)),
                                        name: Some(tc.function.name),
                                        arguments_delta: Some(tc.function.arguments.to_string()),
                                    },
                                    finish_reason: if is_done { Some(FinishReason::ToolCalls) } else { None },
                                    usage: usage.clone(),
                                });
                            }
                        }

                        if !resp.message.content.is_empty() {
                            yield Ok(StreamChunk {
                                delta: StreamChunkDelta::Text(resp.message.content),
                                finish_reason: if is_done { Some(FinishReason::Stop) } else { None },
                                usage,
                            });
                        } else if is_done {
                            yield Ok(StreamChunk::done(FinishReason::Stop, usage));
                        }
                    }
                    Err(e) => {
                        tracing::debug!("Failed to parse Ollama line: {}", e);
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}
