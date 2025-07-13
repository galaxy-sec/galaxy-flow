mod dict;
use crate::infra::once_init_log;

use super::context::ExecContext;

pub mod action;
pub mod global;
pub mod hold;
pub mod job;
pub mod runnable;
pub mod sequence;
pub mod trans;
pub use dict::DictUse;
pub use dict::VarSpace;
pub mod task;
pub mod unit;

#[allow(dead_code)]
pub fn exec_init_env() -> (ExecContext, VarSpace) {
    once_init_log();
    let ctx = ExecContext::new(Some(false), false);
    let def = VarSpace::default();
    (ctx, def)
}
