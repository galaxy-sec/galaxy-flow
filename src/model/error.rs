use derive_more::From;
use orion_conf::error::SerdeReason;
use orion_error::{DomainReason, ErrorCode, StructError, UvsReason};
use orion_sec::{OrionSecReason, SecReason};
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, PartialEq, Serialize, From, Error)]
pub enum AssembleReason {
    #[error("miss : {0}")]
    Miss(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

impl DomainReason for AssembleReason {}

impl ErrorCode for AssembleReason {
    fn error_code(&self) -> i32 {
        520
    }
}

pub type AssembleError = StructError<AssembleReason>;
pub type AResult<T> = Result<T, AssembleError>;

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum ExecReason {
    #[error("cmd err : {1},{2}")]
    OsCmd(String, i32, String),
    #[error("io err : {0}")]
    Io(String),
    #[error("gxl : {0}")]
    Gxl(String),
    #[error("serv: {0}")]
    Serv(String),
    #[error("assert fail! : {0}")]
    Assert(String),
    #[error("args err : {0}")]
    Args(String),
    #[error("miss : {0}")]
    Miss(String),
    #[error("serde err : {0}")]
    Serde(String),
    #[error("{0}")]
    Uvs(UvsReason),
    #[error("{0}")]
    Sec(SecReason),

    #[error("{0}")]
    NetWork(String),
}

impl DomainReason for ExecReason {}
impl From<UvsReason> for ExecReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}
impl ErrorCode for ExecReason {
    fn error_code(&self) -> i32 {
        510
    }
}

impl From<reqwest::Error> for ExecReason {
    fn from(value: reqwest::Error) -> Self {
        ExecReason::NetWork(value.to_string())
    }
}

pub type ExecError = StructError<ExecReason>;
pub type ExecResult<T> = Result<T, ExecError>;

impl From<SerdeReason> for ExecReason {
    fn from(value: SerdeReason) -> Self {
        ExecReason::Serde(format!("Serde error: {value}"))
    }
}

impl From<OrionSecReason> for ExecReason {
    fn from(value: OrionSecReason) -> Self {
        match value {
            OrionSecReason::Sec(sec_reason) => Self::Sec(sec_reason),
            OrionSecReason::Uvs(uvs_reason) => Self::Uvs(map_legacy_uvs_reason(&uvs_reason)),
        }
    }
}

fn map_legacy_uvs_reason(value: &impl std::fmt::Debug) -> UvsReason {
    let debug = format!("{value:?}");
    match debug.as_str() {
        "ValidationError" => UvsReason::ValidationError,
        "BusinessError" => UvsReason::BusinessError,
        "RunRuleError" => UvsReason::RunRuleError,
        "NotFoundError" => UvsReason::NotFoundError,
        "PermissionError" => UvsReason::PermissionError,
        "DataError" => UvsReason::DataError,
        "SystemError" => UvsReason::SystemError,
        "NetworkError" => UvsReason::NetworkError,
        "ResourceError" => UvsReason::ResourceError,
        "TimeoutError" => UvsReason::TimeoutError,
        "ExternalError" => UvsReason::ExternalError,
        "LogicError" => UvsReason::LogicError,
        "ConfigError(Core)" => UvsReason::core_conf(),
        "ConfigError(Feature)" => UvsReason::feature_conf(),
        "ConfigError(Dynamic)" => UvsReason::dynamic_conf(),
        _ => UvsReason::SystemError,
    }
}
