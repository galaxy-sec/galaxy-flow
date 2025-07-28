use crate::{calculate::dynval::EvalError, context::ExecContext, execution::VarSpace};

pub trait WildEq<Rhs: ?Sized = Self> {
    fn we(&self, other: &Rhs) -> bool;
}
pub trait Evaluation {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult;
}
pub type DecideResult = Result<bool, EvalError>;
pub trait EvalArgs {}
impl EvalArgs for u32 {}
impl EvalArgs for () {}
