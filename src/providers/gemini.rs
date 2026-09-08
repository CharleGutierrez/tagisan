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

pub struct GeminiProvider {
    api_key: String,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            client: reqwest::Client::new(),
        }
    }

    fn sanitize_model(model: &str) -> String {
        if model.starts_with("models/") {
            model.to_string()
        } else {
            format!("models/{}", model)
        }
    }
}

#[derive(Serialize)]
struct GeminiGeneratePayload {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: Option<String>,
}

#[derive(Serialize)]
struct GeminiGenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output_tokens: Option<u32>,
}

#[derive(Serialize)]
struct GeminiSystemInstruction {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiApiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    usage_metadata: Option<GeminiUsageMetadata>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
    finish_reason: Option<String>,
}

#[derive(Deserialize)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize)]
struct GeminiUsageMetadata {
    prompt_token_count: Option<u32>,
    candidates_token_count: Option<u32>,
}

#[async_trait]
impl LlmProvider for GeminiProvider {
    fn provider_id(&self) -> &'static str {
        "gemini"
    }

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        let m = model.to_lowercase();
        let mut caps = ProviderCapabilities::STREAMING
            | ProviderCapabilities::SYSTEM_PROMPT
            | ProviderCapabilities::FUNCTION_CALLING
            | ProviderCapabilities::VISION;

        if m.contains("2.0") || m.contains("flash") {
            caps |= ProviderCapabilities::PROMPT_CACHING;
        }
        caps
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let model_clean = Self::sanitize_model(&req.model);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}:generateContent?key={}",
            model_clean, self.api_key
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut contents = Vec::new();
        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "model",
                _ => "user",
            };
            contents.push(GeminiContent {
                role: role_str.to_string(),
                parts: vec![GeminiPart {
                    text: Some(msg.extract_text()),
                }],
            });
        }

        let system_instruction = req.system_prompt.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiPart { text: Some(s) }],
        });

        let payload = GeminiGeneratePayload {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            }),
            system_instruction,
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
                return Err(TagisanError::RateLimited("gemini".into(), None));
            }
            if status.as_u16() == 400 || status.as_u16() == 403 {
                return Err(TagisanError::Authentication("gemini".into(), err_text));
            }
            return Err(TagisanError::BadResponse("gemini".into(), err_text));
        }

        let api_resp: GeminiApiResponse = response.json().await?;
        let candidate = api_resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .ok_or_else(|| TagisanError::BadResponse("gemini".into(), "No candidates returned".into()))?;

        let mut extracted_text = String::new();
        if let Some(content) = candidate.content {
            for part in content.parts {
                if let Some(t) = part.text {
                    extracted_text.push_str(&t);
                }
            }
        }

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => FinishReason::Stop,
            Some("MAX_TOKENS") => FinishReason::Length,
            Some(other) => FinishReason::Other(other.to_string()),
            None => FinishReason::Stop,
        };

        let usage = match api_resp.usage_metadata {
            Some(u) => TokenUsage {
                prompt_tokens: u.prompt_token_count.unwrap_or(0),
                completion_tokens: u.candidates_token_count.unwrap_or(0),
                reasoning_tokens: None,
                cached_prompt_tokens: None,
                estimated_cost_usd: None,
            },
            None => TokenUsage::default(),
        };

        Ok(CompletionResponse {
            id: format!("gemini_{}", start.elapsed().as_millis()),
            provider: "gemini".to_string(),
            model: req.model,
            message: Message {
                role: Role::Assistant,
                content: vec![ContentBlock::Text { text: extracted_text }],
                name: None,
                metadata: std::collections::HashMap::new(),
            },
            finish_reason,
            usage,
            latency: start.elapsed(),
        })
    }

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        let model_clean = Self::sanitize_model(&req.model);
        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/{}:streamGenerateContent?alt=sse&key={}",
            model_clean, self.api_key
        );

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let mut contents = Vec::new();
        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "model",
                _ => "user",
            };
            contents.push(GeminiContent {
                role: role_str.to_string(),
                parts: vec![GeminiPart {
                    text: Some(msg.extract_text()),
                }],
            });
        }

        let system_instruction = req.system_prompt.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiPart { text: Some(s) }],
        });

        let payload = GeminiGeneratePayload {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            }),
            system_instruction,
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
                return Err(TagisanError::RateLimited("gemini".into(), None));
            }
            return Err(TagisanError::BadResponse("gemini".into(), err_text));
        }

        let event_stream = response.bytes_stream().eventsource();

        let mapped = event_stream.filter_map(|event_res| async move {
            match event_res {
                Ok(event) => {
                    let data = event.data.trim();
                    if data.is_empty() {
                        return None;
                    }

                    match serde_json::from_str::<GeminiApiResponse>(data) {
                        Ok(resp) => {
                            let usage = resp.usage_metadata.map(|u| TokenUsage {
                                prompt_tokens: u.prompt_token_count.unwrap_or(0),
                                completion_tokens: u.candidates_token_count.unwrap_or(0),
                                reasoning_tokens: None,
                                cached_prompt_tokens: None,
                                estimated_cost_usd: None,
                            });

                            if let Some(candidate) = resp.candidates.and_then(|c| c.into_iter().next()) {
                                if let Some(content) = candidate.content {
                                    for part in content.parts {
                                        if let Some(text) = part.text {
                                            if !text.is_empty() {
                                                return Some(Ok(StreamChunk {
                                                    delta: StreamChunkDelta::Text(text),
                                                    finish_reason: candidate.finish_reason.map(|_| FinishReason::Stop),
                                                    usage,
                                                }));
                                            }
                                        }
                                    }
                                }
                            }
                            None
                        }
                        Err(e) => {
                            tracing::debug!("Failed to parse Gemini stream chunk: {}", e);
                            None
                        }
                    }
                }
                Err(e) => Some(Err(TagisanError::BadResponse("gemini".into(), e.to_string()))),
            }
        });

        Ok(Box::pin(mapped))
    }
}
