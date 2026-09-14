use crate::auth::GeminiOAuthManager;
use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message,
    ProviderCapabilities, Role, StreamChunk, StreamChunkDelta, TokenUsage,
};
use async_trait::async_trait;
use eventsource_stream::Eventsource;
use futures::StreamExt;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

#[derive(Clone)]
pub enum GeminiAuth {
    ApiKey(String),
    OAuth(Arc<Mutex<GeminiOAuthManager>>),
}

pub struct GeminiProvider {
    auth: GeminiAuth,
    client: reqwest::Client,
}

impl GeminiProvider {
    pub fn new(api_key: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            auth: GeminiAuth::ApiKey(api_key.into()),
            client,
        }
    }

    pub fn with_oauth(oauth_manager: GeminiOAuthManager) -> Self {
        let client = reqwest::Client::builder()
            .connect_timeout(std::time::Duration::from_secs(10))
            .timeout(std::time::Duration::from_secs(180))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            auth: GeminiAuth::OAuth(Arc::new(Mutex::new(oauth_manager))),
            client,
        }
    }

    /// Check if either API key or OAuth session is present and available
    pub fn is_available() -> bool {
        if let Ok(key) = std::env::var("GEMINI_API_KEY") {
            let k = key.trim();
            if !k.is_empty() && !k.starts_with("your_") && !k.ends_with("_key_here") {
                return true;
            }
        }
        GeminiOAuthManager::is_authenticated()
    }

    /// Alias for is_available()
    pub fn has_credentials() -> bool {
        Self::is_available()
    }

    /// Helper to map model names when using OAuth / AntiGravity endpoint
    pub fn sanitize_oauth_model(model: &str) -> &str {
        let clean = model.strip_prefix("models/").unwrap_or(model);
        if clean.is_empty() || clean == "auto" || clean.contains("2.0") || clean.contains("1.5") {
            "gemini-2.5-flash"
        } else {
            clean
        }
    }

    /// Prepare the API URL and authorization headers dynamically based on auth type
    async fn prepare_request_auth(&self, model: &str, is_stream: bool) -> Result<(String, HeaderMap)> {
        let model_clean = Self::sanitize_model(model);
        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        let stream_param = if is_stream {
            "streamGenerateContent?alt=sse"
        } else {
            "generateContent"
        };

        match &self.auth {
            GeminiAuth::ApiKey(key) => {
                let separator = if is_stream { "&" } else { "?" };
                let url = format!(
                    "https://generativelanguage.googleapis.com/v1beta/{}:{}{}key={}",
                    model_clean, stream_param, separator, key
                );
                Ok((url, headers))
            }
            GeminiAuth::OAuth(mgr) => {
                let token = {
                    let mut lock = mgr.lock().await;
                    lock.get_valid_access_token().await?
                };
                let url = format!(
                    "https://daily-cloudcode-pa.googleapis.com/v1internal:{}",
                    stream_param
                );
                let auth_val = HeaderValue::from_str(&format!("Bearer {}", token))
                    .map_err(|e| TagisanError::Authentication("gemini".into(), format!("Invalid auth header: {e}")))?;
                headers.insert(AUTHORIZATION, auth_val);
                headers.insert(
                    USER_AGENT,
                    HeaderValue::from_static("antigravity/2.0.0"),
                );
                Ok((url, headers))
            }
        }
    }

    fn sanitize_model(model: &str) -> String {
        if model.starts_with("models/") {
            model.to_string()
        } else {
            format!("models/{}", model)
        }
    }

    fn format_contents(&self, req: &CompletionRequest) -> Vec<GeminiContent> {
        let mut tool_id_to_name: std::collections::HashMap<String, String> = std::collections::HashMap::new();
        for msg in &req.messages {
            for block in &msg.content {
                if let ContentBlock::ToolCall { id, name, .. } = block {
                    tool_id_to_name.insert(id.clone(), name.clone());
                }
            }
        }

        let mut contents: Vec<GeminiContent> = Vec::new();

        for msg in &req.messages {
            let role_str = match msg.role {
                Role::User => "user",
                Role::Assistant => "model",
                Role::Tool => "user", // In Gemini API v1beta, functionResponse parts must reside in 'user' role
                Role::Reasoning => "model",
                Role::System => "user",
            };

            let mut parts = Vec::new();

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        parts.push(GeminiPart {
                            text: Some(text.clone()),
                            ..Default::default()
                        });
                    }
                    ContentBlock::Thinking { thinking, .. } => {
                        parts.push(GeminiPart {
                            text: Some(format!("<think>\n{}\n</think>", thinking)),
                            ..Default::default()
                        });
                    }
                    ContentBlock::Image { media_type, data_base64 } => {
                        parts.push(GeminiPart {
                            inline_data: Some(GeminiBlob {
                                mime_type: media_type.clone(),
                                data: data_base64.clone(),
                            }),
                            ..Default::default()
                        });
                    }
                    ContentBlock::ToolCall { name, arguments, .. } => {
                        parts.push(GeminiPart {
                            function_call: Some(GeminiFunctionCall {
                                name: name.clone(),
                                args: arguments.clone(),
                            }),
                            ..Default::default()
                        });
                    }
                    ContentBlock::ToolResult { tool_call_id, content, is_error } => {
                        let fn_name = tool_id_to_name
                            .get(tool_call_id)
                            .cloned()
                            .unwrap_or_else(|| {
                                if let Some((name, _)) = tool_call_id.split_once(':') {
                                    name.to_string()
                                } else {
                                    tool_call_id.clone()
                                }
                            });

                        parts.push(GeminiPart {
                            function_response: Some(GeminiFunctionResponse {
                                name: fn_name,
                                response: serde_json::json!({
                                    "result": content,
                                    "is_error": is_error
                                }),
                            }),
                            ..Default::default()
                        });
                    }
                }
            }

            if let Some(last) = contents.last_mut() {
                if last.role == role_str {
                    last.parts.extend(parts);
                    continue;
                }
            }

            contents.push(GeminiContent {
                role: role_str.to_string(),
                parts,
            });
        }

        contents
    }
}

#[derive(Serialize)]
struct GeminiGeneratePayload<'a> {
    contents: Vec<GeminiContent>,
    #[serde(skip_serializing_if = "Option::is_none")]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiSystemInstruction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<GeminiToolWrapper<'a>>>,
}

#[derive(Serialize)]
struct GeminiCcpaPayload<'a> {
    project: &'static str,
    model: &'a str,
    request: GeminiGeneratePayload<'a>,
}

#[derive(Serialize)]
struct GeminiToolWrapper<'a> {
    #[serde(rename = "functionDeclarations")]
    function_declarations: Vec<GeminiFunctionDeclaration<'a>>,
}

#[derive(Serialize)]
struct GeminiFunctionDeclaration<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiContent {
    role: String,
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
struct GeminiPart {
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
    #[serde(default)]
    thought: Option<bool>,
    #[serde(rename = "thoughtSignature", default)]
    thought_signature: Option<String>,
    #[serde(rename = "inlineData", skip_serializing_if = "Option::is_none")]
    inline_data: Option<GeminiBlob>,
    #[serde(rename = "functionCall", skip_serializing_if = "Option::is_none")]
    function_call: Option<GeminiFunctionCall>,
    #[serde(rename = "functionResponse", skip_serializing_if = "Option::is_none")]
    function_response: Option<GeminiFunctionResponse>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiBlob {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiFunctionCall {
    name: String,
    args: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GeminiFunctionResponse {
    name: String,
    response: serde_json::Value,
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

#[derive(Deserialize, Debug)]
struct GeminiApiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
    #[serde(rename = "usageMetadata")]
    usage_metadata: Option<GeminiUsageMetadata>,
    #[serde(default)]
    response: Option<Box<GeminiApiResponse>>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidate {
    content: Option<GeminiCandidateContent>,
    #[serde(rename = "finishReason")]
    finish_reason: Option<String>,
}

#[derive(Deserialize, Debug)]
struct GeminiCandidateContent {
    parts: Vec<GeminiPart>,
}

#[derive(Deserialize, Debug)]
struct GeminiUsageMetadata {
    #[serde(rename = "promptTokenCount")]
    prompt_token_count: Option<u32>,
    #[serde(rename = "candidatesTokenCount")]
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
        let (url, headers) = self.prepare_request_auth(&req.model, false).await?;

        let contents = self.format_contents(&req);

        let system_instruction = req.system_prompt.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiPart {
                text: Some(s),
                ..Default::default()
            }],
        });

        let tools = if !req.tools.is_empty() {
            Some(vec![GeminiToolWrapper {
                function_declarations: req
                    .tools
                    .iter()
                    .map(|t| GeminiFunctionDeclaration {
                        name: &t.name,
                        description: &t.description,
                        parameters: &t.parameters,
                    })
                    .collect(),
            }])
        } else {
            None
        };

        let payload = GeminiGeneratePayload {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            }),
            system_instruction,
            tools,
        };

        let is_oauth = matches!(&self.auth, GeminiAuth::OAuth(_));
        let body_value = if is_oauth {
            let oauth_model = Self::sanitize_oauth_model(&req.model);
            serde_json::to_value(&GeminiCcpaPayload {
                project: "default-cli-project",
                model: oauth_model,
                request: payload,
            })
        } else {
            serde_json::to_value(&payload)
        }
        .map_err(TagisanError::Serialization)?;

        let send_future = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body_value)
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
                return Err(TagisanError::RateLimited("gemini".into(), None));
            }
            if status.as_u16() == 400 || status.as_u16() == 403 {
                return Err(TagisanError::Authentication("gemini".into(), err_text));
            }
            return Err(TagisanError::BadResponse("gemini".into(), err_text));
        }

        let api_resp: GeminiApiResponse = response.json().await?;
        let (candidates, usage_metadata) = if let Some(inner) = api_resp.response {
            (inner.candidates, inner.usage_metadata)
        } else {
            (api_resp.candidates, api_resp.usage_metadata)
        };

        let candidate = candidates
            .and_then(|c| c.into_iter().next())
            .ok_or_else(|| TagisanError::BadResponse("gemini".into(), "No candidates returned".into()))?;

        let mut content_blocks = Vec::new();
        let mut has_tool_calls = false;

        if let Some(content) = candidate.content {
            for (idx, part) in content.parts.into_iter().enumerate() {
                if let Some(t) = part.text {
                    content_blocks.push(ContentBlock::Text { text: t });
                }
                if let Some(fc) = part.function_call {
                    has_tool_calls = true;
                    content_blocks.push(ContentBlock::ToolCall {
                        id: format!("{}:gemini_{}", fc.name, idx),
                        name: fc.name,
                        arguments: fc.args,
                    });
                }
            }
        }

        let finish_reason = match candidate.finish_reason.as_deref() {
            Some("STOP") => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
            Some("MAX_TOKENS") => FinishReason::Length,
            Some("FUNCTION_CALL") => FinishReason::ToolCalls,
            Some(other) => FinishReason::Other(other.to_string()),
            None => {
                if has_tool_calls {
                    FinishReason::ToolCalls
                } else {
                    FinishReason::Stop
                }
            }
        };

        let usage = match usage_metadata {
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
        let (url, headers) = self.prepare_request_auth(&req.model, true).await?;

        let contents = self.format_contents(&req);

        let system_instruction = req.system_prompt.map(|s| GeminiSystemInstruction {
            parts: vec![GeminiPart {
                text: Some(s),
                ..Default::default()
            }],
        });

        let tools = if !req.tools.is_empty() {
            Some(vec![GeminiToolWrapper {
                function_declarations: req
                    .tools
                    .iter()
                    .map(|t| GeminiFunctionDeclaration {
                        name: &t.name,
                        description: &t.description,
                        parameters: &t.parameters,
                    })
                    .collect(),
            }])
        } else {
            None
        };

        let payload = GeminiGeneratePayload {
            contents,
            generation_config: Some(GeminiGenerationConfig {
                temperature: req.temperature,
                max_output_tokens: req.max_tokens,
            }),
            system_instruction,
            tools,
        };

        let is_oauth = matches!(&self.auth, GeminiAuth::OAuth(_));
        let body_value = if is_oauth {
            let oauth_model = Self::sanitize_oauth_model(&req.model);
            serde_json::to_value(&GeminiCcpaPayload {
                project: "default-cli-project",
                model: oauth_model,
                request: payload,
            })
        } else {
            serde_json::to_value(&payload)
        }
        .map_err(TagisanError::Serialization)?;

        let send_future = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body_value)
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
                return Err(TagisanError::RateLimited("gemini".into(), None));
            }
            return Err(TagisanError::BadResponse("gemini".into(), err_text));
        }

        let mut event_stream = response.bytes_stream().eventsource();
        let cancellation_token = req.cancellation_token.clone();

        let output_stream = async_stream::stream! {
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
                        if data.starts_with("{\"error\":") || data.starts_with("[{\"error\":") {
                            tracing::warn!("Gemini stream error payload: {}", data);
                            continue;
                        }

                        match serde_json::from_str::<GeminiApiResponse>(data) {
                            Ok(resp) => {
                                let (candidates, usage_metadata) = if let Some(inner) = resp.response {
                                    (inner.candidates, inner.usage_metadata)
                                } else {
                                    (resp.candidates, resp.usage_metadata)
                                };

                                let usage = usage_metadata.map(|u| TokenUsage {
                                    prompt_tokens: u.prompt_token_count.unwrap_or(0),
                                    completion_tokens: u.candidates_token_count.unwrap_or(0),
                                    reasoning_tokens: None,
                                    cached_prompt_tokens: None,
                                    estimated_cost_usd: None,
                                });

                                if let Some(candidate) = candidates.and_then(|c| c.into_iter().next()) {
                                    if let Some(content) = candidate.content {
                                        for (idx, part) in content.parts.into_iter().enumerate() {
                                            if let Some(text) = part.text {
                                                if !text.is_empty() {
                                                    yield Ok(StreamChunk {
                                                        delta: StreamChunkDelta::Text(text),
                                                        finish_reason: candidate.finish_reason.as_deref().map(|r| match r {
                                                            "FUNCTION_CALL" => FinishReason::ToolCalls,
                                                            _ => FinishReason::Stop,
                                                        }),
                                                        usage: usage.clone(),
                                                    });
                                                }
                                            }
                                            if let Some(fc) = part.function_call {
                                                yield Ok(StreamChunk {
                                                    delta: StreamChunkDelta::ToolCallDelta {
                                                        index: idx,
                                                        id: Some(format!("{}:gemini_{}", fc.name, idx)),
                                                        name: Some(fc.name),
                                                        arguments_delta: Some(fc.args.to_string()),
                                                    },
                                                    finish_reason: Some(FinishReason::ToolCalls),
                                                    usage: usage.clone(),
                                                });
                                            }
                                        }
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::debug!("Failed to parse Gemini stream chunk: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(TagisanError::BadResponse("gemini".into(), e.to_string()));
                    }
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}
