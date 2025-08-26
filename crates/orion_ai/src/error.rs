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
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Execution error: {0}")]
    ExecutionError(String),
    #[error("Tool call error: {0}")]
    ToolCallError(String),
    #[error("Network error: {0}")]
    NetworkError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("Resource not found: {0}")]
    NotFound(String),
    #[error("Internal error: {0}")]
    InternalError(String),
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

/// 错误相关的实用工具函数
pub mod error_utils {
    use super::*;

    /// 创建配置错误
    pub fn config_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::ConfigError(msg.into()))
    }

    /// 创建执行错误
    pub fn execution_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::ExecutionError(msg.into()))
    }

    /// 创建工具调用错误
    pub fn tool_call_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::ToolCallError(msg.into()))
    }

    /// 创建网络错误
    pub fn network_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::NetworkError(msg.into()))
    }

    /// 创建序列化错误
    pub fn serialization_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::SerializationError(msg.into()))
    }

    /// 创建反序列化错误
    pub fn deserialization_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::DeserializationError(msg.into()))
    }

    /// 创建无效输入错误
    pub fn invalid_input_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::InvalidInput(msg.into()))
    }

    /// 创建权限错误
    pub fn permission_denied_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::PermissionDenied(msg.into()))
    }

    /// 创建资源未找到错误
    pub fn not_found_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::NotFound(msg.into()))
    }

    /// 创建内部错误
    pub fn internal_error<S: Into<String>>(msg: S) -> AiError {
        AiError::from(AiErrReason::InternalError(msg.into()))
    }

    /// 检查错误类型
    pub fn is_retryable_error(error: &AiError) -> bool {
        matches!(
            error.reason(),
            AiErrReason::RateLimitError(_)
                | AiErrReason::NetworkError(_)
                | AiErrReason::TokenLimitError(_, _)
        )
    }

    /// 检查是否为用户错误
    pub fn is_user_error(error: &AiError) -> bool {
        matches!(
            error.reason(),
            AiErrReason::InvalidInput(_)
                | AiErrReason::PermissionDenied(_)
                | AiErrReason::SensitiveContentFiltered
        )
    }

    /// 检查是否为配置错误
    pub fn is_config_error(error: &AiError) -> bool {
        matches!(
            error.reason(),
            AiErrReason::ConfigError(_)
                | AiErrReason::NoProviderAvailable
                | AiErrReason::InvalidModel(_)
        )
    }

    /// 格式化错误消息，包含上下文信息
    pub fn format_error_with_context(error: &AiError, context: &str) -> String {
        format!("Context: {}\nError: {}", context, error)
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
            AiErrReason::ConfigError(msg) => {
                UvsReason::core_conf(format!("ai config error: {msg}"))
            }
            AiErrReason::ExecutionError(msg) => {
                UvsReason::DataError(format!("ai execution error: {msg}").into(), None)
            }
            AiErrReason::ToolCallError(msg) => {
                UvsReason::DataError(format!("ai tool call error: {msg}").into(), None)
            }
            AiErrReason::NetworkError(msg) => {
                UvsReason::DataError(format!("network error: {msg}").into(), None)
            }
            AiErrReason::SerializationError(msg) => {
                UvsReason::DataError(format!("serialization error: {msg}").into(), None)
            }
            AiErrReason::DeserializationError(msg) => {
                UvsReason::DataError(format!("deserialization error: {msg}").into(), None)
            }
            AiErrReason::InvalidInput(msg) => {
                UvsReason::validation_error(format!("invalid input: {msg}"))
            }
            AiErrReason::PermissionDenied(msg) => {
                UvsReason::permission_error(format!("permission denied: {msg}"))
            }
            AiErrReason::NotFound(msg) => {
                UvsReason::DataError(format!("resource not found: {msg}").into(), None)
            }
            AiErrReason::InternalError(msg) => {
                UvsReason::DataError(format!("internal error: {msg}").into(), None)
            }
            AiErrReason::Uvs(uvs) => uvs,
        }
    }
}
