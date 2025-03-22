use super::prelude::*;
use crate::calculate::cond::{CondExec, IFExpress};
use crate::calculate::dynval::{EnvVarTag, VarCalcSupport};
use crate::calculate::express::EvalArgs;

use super::gxl_block::BlockNode;

#[derive(Clone, Getters, Debug)]
pub struct RgCond {
    pub(crate) cond: IFExpress<BlockNode>,
}

impl RgCond {
    pub fn new(cond: IFExpress<BlockNode>) -> Self {
        Self { cond }
    }
}

impl RunnableTrait for RgCond {
    fn exec(&self, ctx: ExecContext, dct: &mut VarsDict) -> EOResult {
        EnvVarTag::clear_import(&dct.export());
        self.cond.cond_exec(&mut RunArgs {
            ctx,
            def: dct.clone(),
        })
    }
}

#[derive(Debug, Default)]
pub struct RunArgs {
    pub ctx: ExecContext,
    pub def: VarsDict,
}

impl EvalArgs for RunArgs {}
