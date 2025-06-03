use crate::{error::AssembleReason, ExecReason, ExecResult};
use orion_error::{ErrorCode, StructError, UvsBizFrom, UvsReason};
use orion_syspec::error::SpecReason;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, thiserror::Error, PartialEq, Serialize)]
pub enum RunReason {
    #[error("gxl error {0}")]
    Gxl(String),
    #[error("exec error {0}")]
    Exec(String),
    #[error("args error {0}")]
    Args(String),
    #[error("{0}")]
    Uvs(UvsReason),
}
impl From<UvsReason> for RunReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl ErrorCode for RunReason {
    fn error_code(&self) -> i32 {
        match self {
            RunReason::Gxl(_) => 530,
            RunReason::Exec(_) => 540,
            RunReason::Args(_) => 550,
            RunReason::Uvs(uvs_reason) => uvs_reason.error_code(),
        }
    }
}

pub type RunError = StructError<RunReason>;
pub type RunResult<T> = Result<T, RunError>;

#[derive(Debug, PartialEq, Serialize, Error)]
pub enum GxlReason {
    #[error("parse error {0}")]
    Parse(String),
    #[error("depend error {0}")]
    Depend(String),
    #[error("less error {0}")]
    Less(String),
    #[error("none ")]
    None,
    #[error("{0}")]
    Uvs(UvsReason),
}

impl From<UvsReason> for GxlReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

pub type GxlError = StructError<GxlReason>;
pub type GxlResult<T> = std::result::Result<T, GxlError>;
pub type NER = ExecResult<()>;

pub fn report_gxl_error(e: RunError) {
    println!("Galaxy Flow Parse Error (Code: {})", e.error_code());
    println!("--------------------------");
    if let Some(target) = e.target() {
        println!("[TARGET]:\n{}\n", target);
    }
    println!("[REASON]:");
    match e.reason() {
        RunReason::Uvs(uvs_reason) => match uvs_reason {
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
        RunReason::Gxl(e) => {
            println!("GXL ERROR: {}\n", e);
        }
        RunReason::Exec(e) => {
            println!("EXEC ERROR: {}\n", e);
        }
        RunReason::Args(e) => {
            println!("ARGS ERROR: {}\n", e);
        }
    }
    if let Some(pos) = e.position() {
        println!("\n[POSITION]:\n{}", pos);
    }
    if let Some(detail) = e.detail() {
        println!("\n[DETAIL]:\n{}", detail);
    }
    println!("\n[CONTEXT]:\n");
    for x in e.context() {
        println!("{}", x)
    }
}

impl From<ExecReason> for RunReason {
    fn from(value: ExecReason) -> Self {
        match value {
            ExecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => RunReason::Exec(value.to_string()),
        }
    }
}
impl From<RunReason> for ExecReason {
    fn from(value: RunReason) -> Self {
        match value {
            RunReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => Self::Args(value.to_string()),
        }
    }
}

impl From<AssembleReason> for RunReason {
    fn from(value: AssembleReason) -> Self {
        RunReason::Gxl(value.to_string())
    }
}

impl From<SpecReason> for RunReason {
    fn from(value: SpecReason) -> Self {
        match value {
            SpecReason::UnKnow => RunReason::Gxl("unknow".to_string()),
            SpecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            SpecReason::Localize(r) => Self::Uvs(UvsReason::from_biz(r.to_string())),
            SpecReason::Element(r) => Self::Uvs(UvsReason::from_biz(r.to_string())),
        }
    }
}
