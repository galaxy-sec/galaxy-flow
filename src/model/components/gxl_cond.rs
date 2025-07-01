use super::prelude::*;
use crate::calculate::cond::{CondExec, IFExpress};

use super::gxl_block::BlockNode;

#[derive(Clone, Getters, Debug)]
pub struct TGxlCond<T> {
    pub(crate) cond: IFExpress<T>,
}

pub type GxlCond = TGxlCond<BlockNode>;
/*
#[derive(Clone, Getters, Debug)]
pub struct GxlCond {
    pub(crate) cond: IFExpress<BlockNode>,
}
*/

impl GxlCond {
    pub fn new(cond: IFExpress<BlockNode>) -> Self {
        Self { cond }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlCond {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        //EnvVarTag::clear_import(&dct.export());
        self.cond.cond_exec(ctx, dct).await
    }
}
