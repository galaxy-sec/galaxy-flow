use derive_more::From;
use orion_error::{ErrorCode, StructError, UvsReason};
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

pub type ExecError = StructError<ExecReason>;
pub type ExecResult<T> = Result<T, ExecError>;
//pub type ExecResult = Result<ExecOut, Box<dyn std::error::Error>>;
