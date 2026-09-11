use crate::error::{Result, TagisanError};
use crate::providers::{BoxEventStream, LlmProvider};
use crate::types::{CompletionRequest, CompletionResponse, ProviderCapabilities};
use async_trait::async_trait;
use std::sync::Arc;
use tracing::{info, warn};

/// Entry in a CascadeProvider chain
#[derive(Clone)]
pub struct CascadeEntry {
    pub provider: Arc<dyn LlmProvider>,
    pub model: Option<String>,
}

impl CascadeEntry {
    pub fn new(provider: Arc<dyn LlmProvider>, model: Option<String>) -> Self {
        Self { provider, model }
    }
}

/// CascadeProvider implements `LlmProvider` with automatic fallback/failover.
///
/// If the primary provider encounters a retryable failure (such as RateLimited, Network,
/// BadResponse, or Authentication issues), it automatically fails over to the next configured
/// provider in the chain. Non-retryable errors (e.g. BudgetExceeded, Cancelled) halt immediately.
#[derive(Clone)]
pub struct CascadeProvider {
    entries: Vec<CascadeEntry>,
}

impl CascadeProvider {
    /// Create a new CascadeProvider with a list of provider entries
    pub fn new(entries: Vec<CascadeEntry>) -> Self {
        Self { entries }
    }

    /// Create from a list of providers without model overrides
    pub fn from_providers(providers: Vec<Arc<dyn LlmProvider>>) -> Self {
        let entries = providers
            .into_iter()
            .map(|p| CascadeEntry::new(p, None))
            .collect();
        Self { entries }
    }

    /// Add an additional fallback provider to the chain
    pub fn push(&mut self, provider: Arc<dyn LlmProvider>, model: Option<String>) {
        self.entries.push(CascadeEntry::new(provider, model));
    }

    /// Get configured entries
    pub fn entries(&self) -> &[CascadeEntry] {
        &self.entries
    }

    /// Helper to determine if an error qualifies for failover to the next provider.
    /// Allows failover on BudgetExceeded if and only if the next provider is a zero-cost local provider
    /// (e.g. provider_id().to_lowercase() == "ollama" or "local").
    pub fn can_failover(&self, error: &TagisanError, next_entry_idx: usize) -> bool {
        match error {
            TagisanError::RateLimited(_, _) => true,
            TagisanError::BadResponse(_, _) => true,
            TagisanError::Network(_) => true,
            TagisanError::Authentication(_, _) => true,
            TagisanError::ProviderNotFound(_) => true,
            TagisanError::ContextLengthExceeded(_, _, _) => true,
            TagisanError::BudgetExceeded { .. } => {
                if let Some(next_entry) = self.entries.get(next_entry_idx) {
                    let id = next_entry.provider.provider_id().to_lowercase();
                    id == "ollama" || id == "local"
                } else {
                    false
                }
            }
            TagisanError::Cancelled => false,
            TagisanError::Serialization(_) => false,
            TagisanError::Execution(_) => false,
            TagisanError::Io(_) => false,
        }
    }

    /// Helper to determine if an error qualifies for failover to the next provider (generic retryability)
    pub fn is_retryable(error: &TagisanError) -> bool {
        match error {
            TagisanError::RateLimited(_, _) => true,
            TagisanError::BadResponse(_, _) => true,
            TagisanError::Network(_) => true,
            TagisanError::Authentication(_, _) => true,
            TagisanError::ProviderNotFound(_) => true,
            TagisanError::ContextLengthExceeded(_, _, _) => true,
            TagisanError::BudgetExceeded { .. } => false,
            TagisanError::Cancelled => false,
            TagisanError::Serialization(_) => false,
            TagisanError::Execution(_) => false,
            TagisanError::Io(_) => false,
        }
    }
}

#[async_trait]
impl LlmProvider for CascadeProvider {
    fn provider_id(&self) -> &'static str {
        "cascade"
    }

    fn capabilities(&self, model: &str) -> ProviderCapabilities {
        if let Some(first) = self.entries.first() {
            first.provider.capabilities(model)
        } else {
            ProviderCapabilities::STREAMING | ProviderCapabilities::FUNCTION_CALLING
        }
    }

    async fn complete(&self, req: CompletionRequest) -> Result<CompletionResponse> {
        if self.entries.is_empty() {
            return Err(TagisanError::ProviderNotFound(
                "CascadeProvider has no registered providers in its chain".to_string(),
            ));
        }

        let mut last_error = None;

        for (idx, entry) in self.entries.iter().enumerate() {
            let mut current_req = req.clone();
            if let Some(ref m) = entry.model {
                current_req.model = m.clone();
            }

            let provider_id = entry.provider.provider_id();
            let model_name = current_req.model.clone();

            info!(
                "Cascade attempt #{}/{} using provider '{}' (model: '{}')",
                idx + 1,
                self.entries.len(),
                provider_id,
                model_name
            );

            match entry.provider.complete(current_req).await {
                Ok(resp) => {
                    if idx > 0 {
                        info!(
                            "Cascade failover succeeded on fallback provider '{}'",
                            provider_id
                        );
                    }
                    return Ok(resp);
                }
                Err(err) => {
                    warn!(
                        "Provider '{}' failed: {}. Evaluating cascade...",
                        provider_id, err
                    );
                    if !self.can_failover(&err, idx + 1) {
                        // Non-retryable error (e.g. User cancelled or budget limit reached without local fallback)
                        return Err(err);
                    }
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TagisanError::BadResponse(
                "cascade".to_string(),
                "All providers in the cascade chain failed".to_string(),
            )
        }))
    }

    async fn stream(&self, req: CompletionRequest) -> Result<BoxEventStream> {
        if self.entries.is_empty() {
            return Err(TagisanError::ProviderNotFound(
                "CascadeProvider has no registered providers in its chain".to_string(),
            ));
        }

        let mut last_error = None;

        for (idx, entry) in self.entries.iter().enumerate() {
            let mut current_req = req.clone();
            if let Some(ref m) = entry.model {
                current_req.model = m.clone();
            }

            let provider_id = entry.provider.provider_id();
            let model_name = current_req.model.clone();

            info!(
                "Cascade stream attempt #{}/{} using provider '{}' (model: '{}')",
                idx + 1,
                self.entries.len(),
                provider_id,
                model_name
            );

            match entry.provider.stream(current_req).await {
                Ok(stream) => {
                    if idx > 0 {
                        info!(
                            "Cascade stream failover succeeded on fallback provider '{}'",
                            provider_id
                        );
                    }
                    return Ok(stream);
                }
                Err(err) => {
                    warn!(
                        "Provider '{}' stream initialization failed: {}. Evaluating cascade...",
                        provider_id, err
                    );
                    if !self.can_failover(&err, idx + 1) {
                        return Err(err);
                    }
                    last_error = Some(err);
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            TagisanError::BadResponse(
                "cascade".to_string(),
                "All providers in the cascade chain failed to initialize stream".to_string(),
            )
        }))
    }
}
