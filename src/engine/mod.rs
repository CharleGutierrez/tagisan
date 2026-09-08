pub mod budget;

use crate::error::{Result, TagisanError};
use crate::providers::LlmProvider;
use budget::TokenBudgetTracker;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_util::sync::CancellationToken;

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
}
