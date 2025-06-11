use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
use orion_syspec::error::SpecReason;
use serde::Serialize;
use thiserror::Error;

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
    //#[display("cmd: {_0}")]
    OsCmd(String, i32, String),
    #[error("io err : {0}")]
    Io(String),
    #[error("check err : {0}")]
    Check(String),
    #[error("args err : {0}")]
    Args(String),
    #[error("depend err : {0}")]
    Depend(String),
    #[error("exp err : {0}")]
    Exp(String),
    #[error("bug : {0}")]
    Bug(String),
    #[error("no val: {0}")]
    NoVal(String),
    #[error("miss : {0}")]
    Miss(String),
    #[error("{0}")]
    Uvs(UvsReason),
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

impl From<SpecReason> for ExecReason {
    fn from(value: SpecReason) -> Self {
        match value {
            SpecReason::UnKnow => todo!(),
            SpecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            SpecReason::Localize(r) => Self::Depend(r.to_string()),
            SpecReason::Element(r) => Self::Depend(r.to_string()),
        }
    }
}
