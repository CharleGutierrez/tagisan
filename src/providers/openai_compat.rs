use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use crate::types::{
    CompletionRequest, CompletionResponse, ContentBlock, FinishReason, Message, Role, TokenUsage,
};
use async_trait::async_trait;
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

    /// Factory for OpenAI
    pub fn openai(api_key: impl Into<String>) -> Self {
        Self::new("openai", "https://api.openai.com/v1", api_key)
    }

    /// Factory for xAI (Grok)
    pub fn xai(api_key: impl Into<String>) -> Self {
        Self::new("xai", "https://api.x.ai/v1", api_key)
    }

    /// Factory for DeepSeek
    pub fn deepseek(api_key: impl Into<String>) -> Self {
        Self::new("deepseek", "https://api.deepseek.com", api_key)
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
struct ApiUsage {
    prompt_tokens: Option<u32>,
    completion_tokens: Option<u32>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn provider_id(&self) -> &'static str {
        self.provider_id
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        let start = Instant::now();
        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.api_key))
                .map_err(|e| TagisanError::Authentication(self.provider_id.to_string(), e.to_string()))?,
        );

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

        // Check if reasoning content is provided (DeepSeek R1 / OpenAI o1/o3)
        if let Some(reasoning) = choice.message.reasoning_content {
            if !reasoning.trim().is_empty() {
                content_blocks.push(ContentBlock::Thinking {
                    thinking: reasoning,
                    signature: None,
                });
            }
        }

        let raw_text = choice.message.content.unwrap_or_default();

        // Parse embedded <think>...</think> if returned inside content text (e.g. DeepSeek R1)
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
}
