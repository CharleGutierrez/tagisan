pub mod budget;

use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use budget::TokenBudgetTracker;
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
        for name in &["anthropic", "openai", "gemini", "xai", "deepseek", "ollama"] {
            if let Some(p) = self.providers.get(*name) {
                return Some(p.clone());
            }
        }
        self.providers.values().next().cloned()
    }
}
