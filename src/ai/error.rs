use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("Network request failed: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("API authentication failed for provider: {0}")]
    AuthError(String),

    #[error("API rate limit exceeded for provider: {0}")]
    RateLimitError(String),

    #[error("Token limit exceeded: {0} tokens requested, max {1}")]
    TokenLimitError(usize, usize),

    #[error("Context collection failed: {0}")]
    ContextError(String),

    #[error("Git operation failed: {0}")]
    GitError(#[from] git2::Error),

    #[error("No suitable provider found for request")]
    NoProviderAvailable,

    #[error("Provider timeout after {0}s")]
    TimeoutError(u64),

    #[error("Invalid model specified: {0}")]
    InvalidModel(String),

    #[error("Sensitive content filtered")]
    SensitiveContentFiltered,
}

impl AiError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            AiError::NetworkError(_) | AiError::RateLimitError(_) | AiError::TimeoutError(_)
        )
    }

    pub fn provider_name(&self) -> Option<&str> {
        match self {
            AiError::AuthError(provider)
            | AiError::RateLimitError(provider)
            | AiError::InvalidModel(provider) => Some(provider),
            _ => None,
        }
    }
}

pub type AiResult<T> = Result<T, AiError>;
