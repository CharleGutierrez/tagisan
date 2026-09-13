pub mod autofix;
pub mod budget;
pub mod embedded;
pub mod gguf;
pub mod server;

pub use autofix::{
    detect_project_type, parse_cargo_json, parse_python_diagnostics, parse_tsc_output,
    AutofixEngine, AutofixOptions, AutofixReport, CompilerDiagnostic, DiagnosticLevel,
    ProjectType,
};

pub use budget::TokenBudgetTracker;
pub use embedded::EmbeddedLlmProvider;
pub use gguf::{
    GgufFile, GgufMetadata, GgufTensorInfo, GgufValue, GgufValueType, OllamaBlobResolver,
    OllamaModelDetails, OllamaModelSummary, DEFAULT_ALIGNMENT, GGUF_MAGIC, GGUF_VERSION_2,
    GGUF_VERSION_3,
};
pub use server::OllamaServer;

use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

#[derive(Clone)]
pub struct EngineContext {
    providers: HashMap<String, Arc<dyn LlmProvider>>,
    pub budget_tracker: Arc<TokenBudgetTracker>,
    pub cancellation_token: CancellationToken,
}

impl EngineContext {
    pub fn new(max_budget_usd: f64) -> Self {
        Self {
            providers: HashMap::new(),
            budget_tracker: Arc::new(TokenBudgetTracker::new(max_budget_usd)),
            cancellation_token: CancellationToken::new(),
        }
    }

    pub fn register_provider(&mut self, provider: Arc<dyn LlmProvider>) {
        self.providers
            .insert(provider.provider_id().to_string(), provider);
    }

    pub fn get_provider(&self, provider_id: &str) -> Result<Arc<dyn LlmProvider>> {
        self.providers
            .get(provider_id)
            .cloned()
            .ok_or_else(|| TagisanError::ProviderNotFound(provider_id.to_string()))
    }

    /// Retrieve the default available provider (checking popular defaults, then any registered provider)
    pub fn default_provider(&self) -> Option<Arc<dyn LlmProvider>> {
        for name in &["embedded", "anthropic", "openai", "gemini", "xai", "deepseek", "ollama"] {
            if let Some(p) = self.providers.get(*name) {
                return Some(p.clone());
            }
        }
        self.providers.values().next().cloned()
    }
}
