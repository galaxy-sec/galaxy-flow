use orion_common::serde::SerdeReason;
use orion_error::{ErrorCode, StructError, UvsConfFrom, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum AiErrReason {
    #[error("API authentication failed for provider: {0}")]
    AuthError(String),

    #[error("API rate limit exceeded for provider: {0}")]
    RateLimitError(String),

    #[error("Token limit exceeded: {0} tokens requested, max {1}")]
    TokenLimitError(usize, usize),

    #[error("Context collection failed: {0}")]
    ContextError(String),

    #[error("No suitable provider found for request")]
    NoProviderAvailable,

    #[error("Invalid model specified: {0}")]
    InvalidModel(String),

    #[error("Sensitive content filtered")]
    SensitiveContentFiltered,
    #[error("{0}")]
    Uvs(UvsReason),
}

impl From<UvsReason> for AiErrReason {
    fn from(value: UvsReason) -> Self {
        AiErrReason::Uvs(value)
    }
}
impl From<SerdeReason> for AiErrReason {
    fn from(value: SerdeReason) -> Self {
        match value {
            SerdeReason::Brief(msg) => Self::Uvs(UvsReason::from_conf(msg)),
            SerdeReason::Uvs(uvs) => Self::Uvs(uvs),
        }
    }
}
impl ErrorCode for AiErrReason {
    fn error_code(&self) -> i32 {
        800
    }
}

impl AiErrReason {
    pub fn provider_name(&self) -> Option<&str> {
        match self {
            AiErrReason::AuthError(provider)
            | AiErrReason::RateLimitError(provider)
            | AiErrReason::InvalidModel(provider) => Some(provider),
            _ => None,
        }
    }
}

pub type AiError = StructError<AiErrReason>;
pub type AiResult<T> = Result<T, AiError>;
