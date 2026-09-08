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
use tokio_util::io::StreamReader;
use tokio::io::AsyncBufReadExt;

pub struct OllamaProvider {
    base_url: String,
    client: reqwest::Client,
}

impl OllamaProvider {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            client: reqwest::Client::new(),
        }
    }

    pub fn default_local() -> Self {
        Self::new("http://localhost:11434")
    }
}

#[derive(Serialize)]
struct OllamaChatPayload<'a> {
    model: &'a str,
    messages: Vec<OllamaMessage<'a>>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage<'a> {
    role: &'a str,
    content: String,
}

#[derive(Deserialize)]
struct OllamaApiResponse {
    message: OllamaMessageResponse,
    done: Option<bool>,
    done_reason: Option<String>,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[derive(Deserialize)]
struct OllamaMessageResponse {
    content: String,
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
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/api/chat", self.base_url.trim_end_matches('/'));

        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(OllamaMessage {
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
            messages.push(OllamaMessage {
                role: role_str,
                content: msg.extract_text(),
            });
        }

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            stream: false,
        };

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let api_resp: OllamaApiResponse = response.json().await?;

        let finish_reason = match api_resp.done_reason.as_deref() {
            Some("stop") => FinishReason::Stop,
            Some("length") => FinishReason::Length,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
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
                content: vec![ContentBlock::Text {
                    text: api_resp.message.content,
                }],
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

        let mut messages = Vec::new();
        if let Some(sys) = &req.system_prompt {
            messages.push(OllamaMessage {
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
            messages.push(OllamaMessage {
                role: role_str,
                content: msg.extract_text(),
            });
        }

        let payload = OllamaChatPayload {
            model: &req.model,
            messages,
            stream: true,
        };

        let response = self.client.post(&url).json(&payload).send().await?;

        if !response.status().is_success() {
            let err_text = response.text().await.unwrap_or_default();
            return Err(TagisanError::BadResponse("ollama".into(), err_text));
        }

        let stream = response.bytes_stream().map(|item| {
            item.map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
        });
        let reader = StreamReader::new(stream);
        let mut lines = reader.lines();

        let output_stream = async_stream::stream! {
            while let Ok(Some(line)) = lines.next_line().await {
                let line_trim = line.trim();
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
