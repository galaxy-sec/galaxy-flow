use async_trait::async_trait;

use super::express::ExpressEnum;
use super::express::*;
use crate::execution::runnable::{EOResult, ExecOut};

use crate::components::gxl_cond::RunArgs;
use std::sync::Arc;
#[async_trait]
pub trait CondExec {
    async fn cond_exec(&self, args: RunArgs) -> EOResult;
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
    async fn cond_exec(&self, args: RunArgs) -> EOResult {
        let x = self.express.decide(&args).expect("express decide error");
        if x {
            self.true_block.cond_exec(args).await
        } else {
            self.false_block.cond_exec(args).await
        }
    }
}
pub struct StuBlock {
    pub out: ExecOut,
}
#[async_trait]
impl CondExec for StuBlock {
    async fn cond_exec(&self, _args: RunArgs) -> EOResult {
        Ok(self.out.clone())
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
            ctrl_express.cond_exec(RunArgs::default()).await,
            Ok(ExecOut::Code(0))
        );
    }
}
