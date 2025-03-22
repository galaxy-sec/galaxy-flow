use super::express::ExpressEnum;
use super::express::*;
use crate::execution::runnable::{EOResult, ExecOut};

use crate::components::gxl_cond::RunArgs;
use std::sync::Arc;
pub trait CondExec {
    fn cond_exec(&self, args: &mut RunArgs) -> EOResult;
}

pub type CondHandle = Arc<dyn CondExec>;

#[derive(Clone, Debug)]
pub struct IFExpress<T> {
    express: ExpressEnum,
    true_block: T,
    false_block: T,
}
impl<T> CondExec for IFExpress<T>
where
    T: CondExec,
{
    fn cond_exec(&self, args: &mut RunArgs) -> EOResult {
        let x = self.express.decide(args).expect("express decide error");
        if x {
            self.true_block.cond_exec(args)
        } else {
            self.false_block.cond_exec(args)
        }
    }
}
pub struct StuBlock {
    pub out: ExecOut,
}
impl CondExec for StuBlock {
    fn cond_exec(&self, _args: &mut RunArgs) -> EOResult {
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
    #[test]
    fn test_ctrl_express() {
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
            ctrl_express.cond_exec(&mut RunArgs::default()),
            Ok(ExecOut::Code(0))
        );
    }
}
