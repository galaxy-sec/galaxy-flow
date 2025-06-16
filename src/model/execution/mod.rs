mod dict;
use crate::infra::once_init_log;

use super::context::ExecContext;

pub mod action;
pub mod global;
pub mod hold;
pub mod job;
pub mod runnable;
pub mod sequence;
pub use dict::DictUse;
pub use dict::VarSpace;
pub mod task;

#[allow(dead_code)]
pub fn exec_init_env() -> (ExecContext, VarSpace) {
    once_init_log();
    let ctx = ExecContext::new(false);
    let def = VarSpace::default();
    (ctx, def)
}
