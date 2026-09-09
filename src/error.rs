use thiserror::Error;

#[derive(Error, Debug)]
pub enum TagisanError {
    #[error("Authentication error for provider '{0}': {1}")]
    Authentication(String, String),

    #[error("Rate limit exceeded for provider '{0}'. Retry after {1:?}")]
    RateLimited(String, Option<std::time::Duration>),

    #[error("Context length exceeded for model '{0}': requested {1} tokens, maximum is {2}")]
    ContextLengthExceeded(String, usize, usize),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Bad or unexpected API response from provider '{0}': {1}")]
    BadResponse(String, String),

    #[error("Provider '{0}' is not registered or found in Engine context")]
    ProviderNotFound(String),

    #[error("Token budget exceeded! Max budget: ${max_budget:.2}, Current: ${current_spent:.4}")]
    BudgetExceeded {
        max_budget: f64,
        current_spent: f64,
    },

    #[error("Operation cancelled by user or supervisor")]
    Cancelled,

    #[error("JSON serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Execution error: {0}")]
    Execution(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl TagisanError {
    /// Determines whether the error is transient and safe to retry automatically
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::BudgetExceeded { .. } => false,
            Self::Authentication(..) => false,
            Self::Cancelled => false,
            Self::ContextLengthExceeded(..) => false,
            Self::ProviderNotFound(..) => false,
            Self::RateLimited(..) => true,
            Self::Network(..) => true,
            Self::BadResponse(..) => true,
            Self::Serialization(..) => false,
            Self::Execution(..) => false,
            Self::Io(..) => false,
        }
    }
}

pub type Result<T> = std::result::Result<T, TagisanError>;
