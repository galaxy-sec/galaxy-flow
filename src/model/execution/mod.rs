use crate::infra::once_init_log;

use super::{context::ExecContext, var::VarsDict};

pub mod hold;
pub mod job;
pub mod runnable;
pub mod sequence;
pub mod task;

#[allow(dead_code)]
pub fn exec_init_env() -> (ExecContext, VarsDict) {
    once_init_log();
    let ctx = ExecContext::new(false);
    let def = VarsDict::default();
    (ctx, def)
}
