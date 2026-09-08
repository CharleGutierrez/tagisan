use crate::error::Result;
use crate::types::{CompletionRequest, CompletionResponse, ProviderCapabilities, StreamChunk};
use async_trait::async_trait;
use futures::stream::BoxStream;

pub mod anthropic;
pub mod cascade;
pub mod gemini;
pub mod ollama;
pub mod openai_compat;

pub type BoxEventStream = BoxStream<'static, Result<StreamChunk>>;

/// Unified Trait implemented by all LLM API adapters
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Provider identifier (e.g. "anthropic", "openai", "xai", "deepseek", "gemini", "ollama", "cascade")
    fn provider_id(&self) -> &'static str;

    /// Return supported capabilities for the given model
    fn capabilities(&self, model: &str) -> ProviderCapabilities;

    /// Execute non-streaming completion request
    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse>;

    /// Execute real-time SSE streaming request
    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream>;
}
