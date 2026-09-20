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
                    id == "ollama" || id == "local" || id == "colibri"
                } else {
                    false
                }
            }
            TagisanError::NoModelsInstalled => true,
            TagisanError::ResourceExhausted(_) => true,
            TagisanError::Cancelled => false,
            TagisanError::Serialization(_) => false,
            TagisanError::Execution(_) => false,
            TagisanError::Security(_) => false,
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
            TagisanError::NoModelsInstalled => false,
            TagisanError::ResourceExhausted(_) => false,
            TagisanError::Cancelled => false,
            TagisanError::Serialization(_) => false,
            TagisanError::Execution(_) => false,
            TagisanError::Security(_) => false,
            TagisanError::Io(_) => false,
        }
    }
}

fn notify_cascade_failover(
    err: &TagisanError,
    current_provider: &str,
    current_model: &str,
    next_entry: &CascadeEntry,
    step_idx: usize,
    total_steps: usize,
) {
    let next_prov_id = next_entry.provider.provider_id();
    let next_model = next_entry.model.as_deref().unwrap_or("default");
    let is_next_local = crate::ecc::is_local_provider(next_prov_id);
    let cost_delta = if is_next_local {
        "+$0.00 (Zero incremental cost on local hardware)"
    } else {
        "Cloud standard consumption"
    };

    let trigger_reason = match err {
        TagisanError::RateLimited(p, wait) => format!(
            "Rate limited (HTTP 429) on '{}'{}",
            p,
            wait.map(|d| format!(" (retry after: {:?})", d)).unwrap_or_default()
        ),
        TagisanError::Authentication(p, msg) => format!("Authentication failure (HTTP 401) on '{}': {}", p, msg),
        TagisanError::BadResponse(p, msg) => format!("Bad response (HTTP 502) on '{}': {}", p, msg),
        TagisanError::Network(msg) => format!("Network failure / timeout: {}", msg),
        TagisanError::BudgetExceeded { max_budget, current_spent } => format!(
            "Token budget reached (${:.2} spent >= ${:.2} max)",
            current_spent, max_budget
        ),
        TagisanError::ContextLengthExceeded(p, req, max) => format!(
            "Context length exceeded on '{}': requested {} > max {}",
            p, req, max
        ),
        TagisanError::ProviderNotFound(p) => format!("Provider not found: {}", p),
        TagisanError::NoModelsInstalled => "No models installed locally in Ollama".to_string(),
        _ => format!("{}", err),
    };

    let action_taken = format!(
        "🔄 Evacuating to fallback provider '{}' ({})",
        next_prov_id, next_model
    );

    crate::notify::notify_failover(
        current_provider,
        current_model,
        next_prov_id,
        next_model,
        &trigger_reason,
        cost_delta,
        &action_taken,
        Some(format!("Cascade step {}/{}", step_idx + 1, total_steps)),
    );
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
                    if let Some(next_entry) = self.entries.get(idx + 1) {
                        notify_cascade_failover(&err, provider_id, &model_name, next_entry, idx, self.entries.len());
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
                    if let Some(next_entry) = self.entries.get(idx + 1) {
                        notify_cascade_failover(&err, provider_id, &model_name, next_entry, idx, self.entries.len());
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
