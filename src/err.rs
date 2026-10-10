use crate::{ExecReason, ExecResult, error::AssembleReason};
use orion_error::reason::UnifiedReason as UvsReason;
use orion_error::{OrionError, StructError};

use serde::Serialize;

#[derive(Debug, PartialEq, Serialize, OrionError)]
pub enum RunReason {
    #[orion_error(identity = "biz.gxl_error")]
    Gxl,
    #[orion_error(identity = "sys.exec_error")]
    Exec,
    #[orion_error(identity = "biz.args_error")]
    Args,
    #[orion_error(identity = "biz.sec_error")]
    Sec,
    #[orion_error(transparent)]
    Uvs(UvsReason),
}

pub type RunError = StructError<RunReason>;
pub type RunResult<T> = Result<T, RunError>;

impl From<UvsReason> for RunReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl RunReason {
    pub fn from_conf() -> Self {
        Self::core_conf()
    }

    pub fn from_sys() -> Self {
        Self::system_error()
    }

    pub fn from_res() -> Self {
        Self::resource_error()
    }
}

#[derive(Debug, PartialEq, Serialize, OrionError)]
pub enum GxlReason {
    #[orion_error(identity = "biz.gxl_parse_error")]
    Parse,
    #[orion_error(identity = "biz.gxl_depend_error")]
    Depend,
    #[orion_error(identity = "biz.gxl_less_error")]
    Less,
    #[orion_error(identity = "biz.gxl_none")]
    None,
    #[orion_error(transparent)]
    Uvs(UvsReason),
}

impl From<UvsReason> for GxlReason {
    fn from(value: UvsReason) -> Self {
        Self::Uvs(value)
    }
}

impl GxlReason {
    pub fn from_logic() -> Self {
        Self::logic_error()
    }
}

pub type GxlError = StructError<GxlReason>;
pub type GxlResult<T> = std::result::Result<T, GxlError>;
pub type NER = ExecResult<()>;

pub fn report_gxl_error(e: RunError) {
    eprintln!("Galaxy Flow Parse Error");
    eprintln!("--------------------------");
    if let Some(target) = e.target_path() {
        eprintln!("[TARGET]:\n{target}\n",);
    }
    // 诊断渲染交给 orion-error 0.9（`DiagnosticReport::render`）：
    // 输出 reason / detail / position / context，替代原先手写的 reason match。
    eprintln!("{}", e.render());
}

impl From<ExecReason> for RunReason {
    fn from(value: ExecReason) -> Self {
        match value {
            ExecReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => RunReason::Exec,
        }
    }
}
impl From<RunReason> for ExecReason {
    fn from(value: RunReason) -> Self {
        match value {
            RunReason::Uvs(uvs_reason) => Self::Uvs(uvs_reason),
            _ => Self::Args,
        }
    }
}

impl From<AssembleReason> for RunReason {
    fn from(_value: AssembleReason) -> Self {
        RunReason::Gxl
    }
}
