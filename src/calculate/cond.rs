use async_trait::async_trait;

use super::express::ExpressEnum;
use super::express::*;
use crate::ability::prelude::{VTResult, VarSpace};
use crate::execution::runnable::ExecOut;

use crate::components::gxl_cond::RunArgs;
use std::sync::Arc;
#[async_trait]
pub trait CondExec {
    async fn cond_exec(&self, def: VarSpace, args: RunArgs) -> VTResult;
}

pub type CondHandle = Arc<dyn CondExec>;

#[derive(Clone, Debug)]
pub struct IFExpress<T> {
    express: ExpressEnum,
    true_block: T,
    false_block: T,
}
#[async_trait]
impl<T> CondExec for IFExpress<T>
where
    T: CondExec + std::marker::Sync,
{
    async fn cond_exec(&self, def: VarSpace, args: RunArgs) -> VTResult {
        let x = self.express.decide(&args).expect("express decide error");
        if x {
            self.true_block.cond_exec(def, args).await
        } else {
            self.false_block.cond_exec(def, args).await
        }
    }
}
pub struct StuBlock {
    pub out: ExecOut,
}
#[async_trait]
impl CondExec for StuBlock {
    async fn cond_exec(&self, _def: VarSpace, _args: RunArgs) -> VTResult {
        Ok((_def, self.out.clone()))
    }
}

impl<T> IFExpress<T>
where
    T: CondExec,
{
    pub(crate) fn new(express: ExpressEnum, true_block: T, false_block: T) -> Self {
        Self {
            express,
            true_block,
            false_block,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::calculate::dynval::MocU32;

    use super::*;
    #[tokio::test]
    async fn test_ctrl_express() {
        let ctrl_express = IFExpress::new(
            ExpressEnum::MU32(BinExpress::eq(MocU32::from("moc_1"), 1)),
            StuBlock {
                out: ExecOut::Code(0),
            },
            StuBlock {
                out: ExecOut::Code(1),
            },
        );
        assert_eq!(
            ctrl_express
                .cond_exec(VarSpace::default(), RunArgs::default())
                .await,
            Ok((VarSpace::default(), ExecOut::Code(0)))
        );
    }
}
