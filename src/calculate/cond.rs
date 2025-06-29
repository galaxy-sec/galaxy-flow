use async_trait::async_trait;

use super::express::ExpressEnum;
use super::express::*;
use crate::ability::prelude::{TaskValue, VTResult, VarSpace};
use crate::components::gxl_cond::TGxlCond;
use crate::context::ExecContext;
use crate::execution::runnable::ExecOut;
use orion_error::ErrorOwe;
use std::sync::Arc;
#[async_trait]
pub trait CondExec {
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult;
}

pub type CondHandle = Arc<dyn CondExec>;

#[derive(Clone, Debug, Getters)]
pub struct IFExpress<T> {
    express: ExpressEnum,
    true_block: T,
    elseif_blocks: Vec<TGxlCond<T>>,
    false_block: Option<T>,
}
#[async_trait]
impl<T> CondExec for IFExpress<T>
where
    T: CondExec + std::marker::Sync,
{
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        let x = self.express.decide(ctx.clone(), &def).owe_logic()?;
        if x {
            self.true_block.cond_exec(ctx, def).await
        } else {
            for cond in &self.elseif_blocks {
                if let Ok(task_value) = cond.cond.cond_exec(ctx.clone(), def.clone()).await {
                    if task_value.rec() != &ExecOut::Ignore {
                        return Ok(task_value);
                    }
                }
            }
            if let Some(false_cond) = self.false_block.as_ref() {
                return false_cond.cond_exec(ctx, def).await;
            }
            Ok(TaskValue::from((def, ExecOut::Ignore)))
        }
    }
}
pub struct StuBlock {
    pub out: ExecOut,
}
#[async_trait]
impl CondExec for StuBlock {
    async fn cond_exec(&self, _ctx: ExecContext, _def: VarSpace) -> VTResult {
        Ok(TaskValue::from((_def, self.out.clone())))
    }
}

impl<T> IFExpress<T>
where
    T: CondExec,
{
    pub(crate) fn new(
        express: ExpressEnum,
        true_block: T,
        elseif_blocks: Vec<TGxlCond<T>>,
        false_block: Option<T>,
    ) -> Self {
        Self {
            express,
            true_block,
            elseif_blocks,
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
            Vec::new(),
            Some(StuBlock {
                out: ExecOut::Code(1),
            }),
        );
        assert_eq!(
            ctrl_express
                .cond_exec(ExecContext::default(), VarSpace::default(),)
                .await,
            Ok(TaskValue::from((VarSpace::default(), ExecOut::Code(0))))
        );
    }

    #[tokio::test]
    async fn test_elseif_blocks() {
        let ctrl_express = IFExpress::new(
            ExpressEnum::MU32(BinExpress::eq(MocU32::from("moc_1"), 0)),
            StuBlock {
                out: ExecOut::Code(0),
            },
            vec![TGxlCond {
                cond: IFExpress::new(
                    ExpressEnum::MU32(BinExpress::eq(MocU32::from("moc_2"), 1)),
                    StuBlock {
                        out: ExecOut::Code(2),
                    },
                    Vec::new(),
                    None,
                ),
            }],
            Some(StuBlock {
                out: ExecOut::Code(1),
            }),
        );
        assert_eq!(
            ctrl_express
                .cond_exec(ExecContext::default(), VarSpace::default(),)
                .await,
            Ok(TaskValue::from((VarSpace::default(), ExecOut::Code(1))))
        );
    }

    #[tokio::test]
    async fn test_false_block() {
        let ctrl_express = IFExpress::new(
            ExpressEnum::MU32(BinExpress::eq(MocU32::from("moc_1"), 0)),
            StuBlock {
                out: ExecOut::Code(0),
            },
            Vec::new(),
            Some(StuBlock {
                out: ExecOut::Code(1),
            }),
        );
        assert_eq!(
            ctrl_express
                .cond_exec(ExecContext::default(), VarSpace::default(),)
                .await,
            Ok(TaskValue::from((VarSpace::default(), ExecOut::Code(1))))
        );
    }
}
