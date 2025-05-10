use std::{self, fmt::Display};

use crate::{error::AssembleReason, ExecReason, ExecResult};
use orion_error::{DomainReason, ErrorCode, StructError, StructReason, UvsReason};
use serde::Serialize;

#[derive(Debug, thiserror::Error, PartialEq, Serialize)]
pub enum RunReason {
    #[error("gxl error {0}")]
    Gxl(String),
    #[error("exec error {0}")]
    Exec(String),
    #[error("args error {0}")]
    Args(String),
}

impl ErrorCode for RunReason {
    fn error_code(&self) -> i32 {
        match self {
            RunReason::Gxl(_) => 530,
            RunReason::Exec(_) => 540,
            RunReason::Args(_) => 550,
        }
    }
}

pub type RunError = StructError<RunReason>;
pub type RunResult<T> = Result<T, RunError>;

impl DomainReason for RunReason {}
#[derive(Debug, PartialEq, Serialize)]
pub enum GxlReason {
    Parse(String),
    Depend(String),
    Less(String),
    None,
}
impl Display for GxlReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GxlReason::Parse(msg) => {
                write!(f, "parse: {}", msg)
            }
            GxlReason::Depend(msg) => {
                write!(f, "depend: {}", msg)
            }
            GxlReason::Less(msg) => {
                write!(f, "less: {}", msg)
            }
            GxlReason::None => todo!(),
        }
    }
}
impl DomainReason for GxlReason {}
pub type GxlError = StructError<GxlReason>;
pub type GxlResult<T> = std::result::Result<T, GxlError>;
pub type NER = ExecResult<()>;

impl From<ExecReason> for StructReason<RunReason> {
    fn from(value: ExecReason) -> Self {
        Self::from(RunReason::Exec(value.to_string()))
    }
}
impl From<AssembleReason> for StructReason<RunReason> {
    fn from(value: AssembleReason) -> Self {
        Self::from(RunReason::Gxl(value.to_string()))
    }
}

pub fn report_rg_error(e: RunError) {
    println!("Galaxy Flow Parse Error (Code: {})", e.error_code());
    println!("--------------------------");
    if let Some(target) = e.target() {
        println!("[TARGET]:\n{}\n", target);
    }
    println!("[REASON]:");
    match e.reason() {
        StructReason::Universal(uvs_reason) => match uvs_reason {
            UvsReason::LogicError(e) => {
                println!("LOGIC ERROR: {}\n", e);
            }
            UvsReason::BizError(e) => {
                println!("BIZ ERROR: {}\n", e);
            }
            UvsReason::DataError(e, _) => {
                println!("DATA ERROR: {}\n", e);
            }
            UvsReason::SysError(e) => {
                println!("SYS ERROR: {}\n", e);
            }
            UvsReason::ResError(e) => {
                println!("RES ERROR: {}\n", e);
            }
            UvsReason::ConfError(e) => {
                println!("CONF ERROR: {}\n", e);
            }
            UvsReason::RuleError(e) => {
                println!("RULE ERROR: {}\n", e);
            }
            UvsReason::PrivacyError(e) => {
                println!("PRIVACY ERROR: {}\n", e);
            }
        },
        StructReason::Domain(domain) => match domain {
            RunReason::Gxl(e) => {
                println!("GXL ERROR: {}\n", e);
            }
            RunReason::Exec(e) => {
                println!("EXEC ERROR: {}\n", e);
            }
            RunReason::Args(e) => {
                println!("ARGS ERROR: {}\n", e);
            }
        },
    }
    if let Some(pos) = e.position() {
        println!("\n[POSITION]:\n{}", pos);
    }
    if let Some(detail) = e.detail() {
        println!("\n[DETAIL]:\n{}", detail);
    }
    println!("\n[CONTEXT]:\n{}", e.context());
}

impl From<ExecReason> for RunReason {
    fn from(value: ExecReason) -> Self {
        RunReason::Exec(value.to_string())
    }
}

impl From<AssembleReason> for RunReason {
    fn from(value: AssembleReason) -> Self {
        RunReason::Gxl(value.to_string())
    }
}
