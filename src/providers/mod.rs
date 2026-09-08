use crate::error::Result;
use crate::types::{CompletionRequest, CompletionResponse};
use async_trait::async_trait;

pub mod anthropic;
pub mod gemini;
pub mod ollama;
pub mod openai_compat;

/// Unified Trait implemented by all LLM API adapters
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identifier (e.g. "anthropic", "openai", "xai", "deepseek", "gemini", "ollama")
    fn provider_id(&self) -> &'static str;

    /// Execute completion request
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;
}
