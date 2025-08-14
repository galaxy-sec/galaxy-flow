use orion_common::serde::SerdeReason;
use orion_error::{ErrorCode, StructError, UvsConfFrom, UvsReason};
use serde_derive::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum AiErrReason {
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

pub type AiError = StructError<AiErrReason>;
pub type AiResult<T> = Result<T, AiError>;

impl From<AiErrReason> for UvsReason {
    fn from(value: AiErrReason) -> Self {
        match value {
            AiErrReason::RateLimitError(msg) => {
                UvsReason::DataError(format!("rate limit {msg}").into(), None)
            }
            AiErrReason::TokenLimitError(limit, max) => {
                UvsReason::DataError(format!("token limit {limit} {max}").into(), None)
            }
            AiErrReason::ContextError(msg) => {
                UvsReason::DataError(format!("ai context error: {msg}").into(), None)
            }
            AiErrReason::NoProviderAvailable => UvsReason::core_conf("no provider"),
            AiErrReason::InvalidModel(msg) => UvsReason::core_conf(format!("invalid model: {msg}")),
            AiErrReason::SensitiveContentFiltered => {
                UvsReason::validation_error("sensitive content filtered")
            }
            AiErrReason::Uvs(uvs) => uvs,
        }
    }
}
