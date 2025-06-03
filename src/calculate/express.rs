use orion_parse::symbol::CmpSymbol;

use crate::context::ExecContext;
use crate::execution::VarSpace;

use super::dynval::{EnvVarTag, EvalError, MocVarTag, ValueEval, VarDef};
use std::fmt::Debug;
use std::num::ParseIntError;

#[derive(Debug, Clone, PartialEq)]
pub enum BinRelation {
    EQ,
    NE,
    GE,
    GT,
    LE,
    LT,
}
#[derive(Clone, Debug)]
pub struct BinExpress<T, E> {
    relation: BinRelation,
    first: T,
    second: E,
}

impl<T, E> BinExpress<T, E> {
    pub fn new(r: BinRelation, first: T, second: E) -> Self {
        BinExpress {
            relation: r,
            first,
            second,
        }
    }
    pub fn eq(first: T, second: E) -> Self {
        BinExpress {
            relation: BinRelation::EQ,
            first,
            second,
        }
    }
    pub fn gt(first: T, second: E) -> Self {
        BinExpress {
            relation: BinRelation::GT,
            first,
            second,
        }
    }

    pub fn le(first: T, second: E) -> Self {
        BinExpress {
            relation: BinRelation::LE,
            first,
            second,
        }
    }
    pub fn from_op(op: CmpSymbol, first: T, second: E) -> Self {
        match op {
            CmpSymbol::We => todo!(),
            CmpSymbol::Eq => Self::new(BinRelation::EQ, first, second),
            CmpSymbol::Ne => todo!(),
            CmpSymbol::Gt => Self::new(BinRelation::GT, first, second),
            CmpSymbol::Ge => todo!(),
            CmpSymbol::Lt => todo!(),
            CmpSymbol::Le => Self::new(BinRelation::LE, first, second),
        }
    }
}
//impl<T, E> ExpressInstance<T, E> for BinExpress<T, E> {}

pub type DecideResult = Result<bool, EvalError>;
pub trait EvalArgs {}
impl EvalArgs for u32 {}
impl EvalArgs for () {}
pub trait Evaluation {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult;
}

impl Evaluation for BinExpress<&str, &str> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first.eq_ignore_ascii_case(self.second),
            BinRelation::NE => !self.first.eq_ignore_ascii_case(self.second),
            BinRelation::GE => self.first.len() > self.second.len(),
            BinRelation::GT => self.first.len() >= self.second.len(),
            BinRelation::LE => self.first.len() < self.second.len(),
            BinRelation::LT => self.first.len() <= self.second.len(),
        })
    }
}

impl Evaluation for BinExpress<String, String> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first == self.second,
            BinRelation::NE => self.first != self.second,
            BinRelation::GE => self.first.len() > self.second.len(),
            BinRelation::GT => self.first.len() >= self.second.len(),
            BinRelation::LE => self.first.len() < self.second.len(),
            BinRelation::LT => self.first.len() <= self.second.len(),
        })
    }
}

impl Evaluation for BinExpress<u32, u32> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first == self.second,
            BinRelation::NE => self.first != self.second,
            BinRelation::GE => self.first > self.second,
            BinRelation::GT => self.first >= self.second,
            BinRelation::LE => self.first < self.second,
            BinRelation::LT => self.first <= self.second,
        })
    }
}

impl<T, E> Evaluation for BinExpress<T, E>
where
    T: ValueEval<E> + Debug + Clone,
    E: PartialEq + PartialOrd,
    //    A: EvalArgs,
{
    fn decide(&self, _ctx: ExecContext, vars_dict: &VarSpace) -> DecideResult {
        let first = self
            .first
            .eval(vars_dict)
            .map_err(|e| EvalError::ValueError(format!("{:?} , e:{}", self.first.clone(), e)))?;
        match self.relation {
            BinRelation::EQ => Ok(first.eq(&self.second)),
            BinRelation::NE => Ok(!first.eq(&self.second)),
            BinRelation::GE => Ok(first.ge(&self.second)),
            BinRelation::GT => Ok(first.gt(&self.second)),
            BinRelation::LE => Ok(first.le(&self.second)),
            BinRelation::LT => Ok(first.lt(&self.second)),
        }
    }
}

impl From<ParseIntError> for EvalError {
    fn from(_: ParseIntError) -> Self {
        EvalError::ParseError
    }
}
pub type BEU32Moc = BinExpress<VarDef<u32, MocVarTag>, u32>;
pub type BEStrMoc = BinExpress<VarDef<String, MocVarTag>, String>;
pub type EVarDef = VarDef<String, EnvVarTag>;
#[derive(Clone, Debug)]
pub enum ExpressEnum {
    EU32(BinExpress<VarDef<u32, EnvVarTag>, u32>),
    EStr(BinExpress<VarDef<String, EnvVarTag>, String>),
    MU32(BinExpress<VarDef<u32, MocVarTag>, u32>),
    MStr(BinExpress<VarDef<String, MocVarTag>, String>),
}

impl Evaluation for ExpressEnum {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult {
        match self {
            ExpressEnum::MU32(x) => x.decide(ctx, args),
            ExpressEnum::MStr(x) => x.decide(ctx, args),
            ExpressEnum::EU32(x) => x.decide(ctx, args),
            ExpressEnum::EStr(x) => x.decide(ctx, args),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::VarSpace;

    use super::*;

    //test bind express
    #[test]
    fn test_bin_express() {
        let bin_express = BinExpress::eq("a", "b");
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
        let bin_express = BinExpress::eq("a", "a");
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
    }
    //test for i32 test bin express
    #[test]
    fn test_bin_express_i32() {
        let bin_express = BinExpress::eq(1, 2);
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
        let bin_express = BinExpress::eq(1, 1);
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
        let bin_express = BinExpress::gt(2, 1);
        assert_eq!(bin_express.relation, BinRelation::GT);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
        let bin_express = BinExpress::gt(1, 2);
        assert_eq!(bin_express.relation, BinRelation::GT);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
    }
    #[test]
    fn test_bin_express_dynstr() {
        type VarMoc = VarDef<String, MocVarTag>;
        let bin_express = BinExpress::eq(VarMoc::from("moc_1"), "1".to_string());
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
        let bin_express = BinExpress::eq(VarMoc::from("moc_1"), "2".to_string());
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
    }
}
