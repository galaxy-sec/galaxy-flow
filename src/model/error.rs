use derive_more::From;
use orion_conf::error::SerdeReason;
use orion_error::{ErrorCode, StructError, UvsReason};
use orion_sec::{OrionSecReason, SecReason};
use serde::Serialize;
use thiserror::Error;

use orion_ai::{AiErrReason, OrionAiReason};

#[derive(Debug, PartialEq, Serialize, From, Error)]
pub enum AssembleReason {
    #[error("miss : {0}")]
    Miss(String),
    #[error("{0}")]
    Uvs(UvsReason),
}

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
    Ai(AiErrReason),

    #[error("{0}")]
    NetWork(String),
}
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

impl From<AiErrReason> for ExecReason {
    fn from(value: AiErrReason) -> Self {
        Self::Ai(value)
    }
}

impl From<OrionAiReason> for ExecReason {
    fn from(value: OrionAiReason) -> Self {
        match value {
            OrionAiReason::Ai(reason) => ExecReason::Ai(reason),
            OrionAiReason::Sec(reason) => ExecReason::Sec(reason),
            OrionAiReason::Uvs(reason) => ExecReason::Uvs(reason),
        }
    }
}
impl From<SerdeReason> for ExecReason {
    fn from(value: SerdeReason) -> Self {
        ExecReason::Serde(format!("Serde error: {value}"))
    }
}

impl From<OrionSecReason> for ExecReason {
    fn from(value: OrionSecReason) -> Self {
        match value {
            OrionSecReason::Sec(sec_reason) => Self::Sec(sec_reason),
            OrionSecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
        }
    }
}
