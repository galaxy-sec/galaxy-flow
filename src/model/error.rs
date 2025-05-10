use orion_error::{DomainReason, ErrorCode, StructError};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub enum AssembleReason {
    Miss(String),
}

impl ErrorCode for AssembleReason {
    fn error_code(&self) -> i32 {
        520
    }
}

impl DomainReason for AssembleReason {}
pub type AssembleError = StructError<AssembleReason>;
pub type AResult<T> = Result<T, AssembleError>;

impl std::fmt::Display for AssembleReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AssembleReason::Miss(msg) => {
                write!(f, "assemble miss: {}", msg)
            }
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum ExecReason {
    OsCmd(String, i32, String),
    Io(String),
    Check(String),
    Args(String),
    Depend(String),
    Exp(String),
    Bug(String),
    NoVal(String),
    Miss(String),
}
impl ErrorCode for ExecReason {
    fn error_code(&self) -> i32 {
        510
    }
}

impl DomainReason for ExecReason {}
pub type ExecError = StructError<ExecReason>;
pub type ExecResult<T> = Result<T, ExecError>;
//pub type ExecResult = Result<ExecOut, Box<dyn std::error::Error>>;
use std::fmt::{self};

impl std::fmt::Display for ExecReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecReason::OsCmd(a, b, c) => write!(
                f,
                "Cmd exec failed!  code:{} \n{:>10}{}\n{:>10}{}",
                b, "prompt: ", a, "at: ", c
            ),
            ExecReason::Io(e) => write!(f, "exec failed : {}", e),
            ExecReason::Check(s)
            | ExecReason::Args(s)
            | ExecReason::Depend(s)
            | ExecReason::Exp(s) => {
                write!(f, "exec failed : {}", s)
            }
            ExecReason::Bug(s) => write!(f, "exec failed have Bug: {}", s),
            ExecReason::NoVal(s) => write!(f, "not found value {}", s),
            ExecReason::Miss(s) => write!(f, "not found {}", s),
        }
    }
}
