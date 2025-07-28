use orion_parse::symbol::CmpSymbol;
use wildmatch::WildMatch;

use crate::calculate::traits::{DecideResult, Evaluation, WildEq};
use crate::context::ExecContext;
use crate::execution::VarSpace;

use super::dynval::{EvalError, ValueEval};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum BinRelation {
    EQ,
    NE,
    GE,
    GT,
    LE,
    LT,
    /// wide match
    WE,
}

#[derive(Clone, Debug)]
pub struct CmpExpress<T, E> {
    pub relation: BinRelation,
    pub first: T,
    pub second: E,
}

impl<T, E> CmpExpress<T, E> {
    pub fn new(r: BinRelation, first: T, second: E) -> Self {
        CmpExpress {
            relation: r,
            first,
            second,
        }
    }
    pub fn eq(first: T, second: E) -> Self {
        CmpExpress {
            relation: BinRelation::EQ,
            first,
            second,
        }
    }
    pub fn gt(first: T, second: E) -> Self {
        CmpExpress {
            relation: BinRelation::GT,
            first,
            second,
        }
    }

    pub fn le(first: T, second: E) -> Self {
        CmpExpress {
            relation: BinRelation::LE,
            first,
            second,
        }
    }
    pub fn from_op(op: CmpSymbol, first: T, second: E) -> Self {
        match op {
            CmpSymbol::Eq => Self::new(BinRelation::EQ, first, second),
            CmpSymbol::Ne => Self::new(BinRelation::NE, first, second),
            CmpSymbol::Gt => Self::new(BinRelation::GT, first, second),
            CmpSymbol::Ge => Self::new(BinRelation::GE, first, second),
            CmpSymbol::Lt => Self::new(BinRelation::LT, first, second),
            CmpSymbol::Le => Self::new(BinRelation::LE, first, second),
            CmpSymbol::We => Self::new(BinRelation::WE, first, second),
        }
    }
}
//impl<T, E> ExpressInstance<T, E> for BinExpress<T, E> {}

impl Evaluation for CmpExpress<&str, &str> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first.eq_ignore_ascii_case(self.second),
            BinRelation::NE => !self.first.eq_ignore_ascii_case(self.second),
            BinRelation::GE => self.first.len() > self.second.len(),
            BinRelation::GT => self.first.len() >= self.second.len(),
            BinRelation::LE => self.first.len() < self.second.len(),
            BinRelation::LT => self.first.len() <= self.second.len(),
            BinRelation::WE => {
                let (patten, value) = if self.first.contains("*") || self.first.contains("?") {
                    (WildMatch::new(self.first), self.second)
                } else {
                    (WildMatch::new(self.second), self.first)
                };
                return Ok(patten.matches(value));
            }
        })
    }
}

impl Evaluation for CmpExpress<String, String> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first == self.second,
            BinRelation::NE => self.first != self.second,
            BinRelation::GE => self.first.len() > self.second.len(),
            BinRelation::GT => self.first.len() >= self.second.len(),
            BinRelation::LE => self.first.len() < self.second.len(),
            BinRelation::LT => self.first.len() <= self.second.len(),
            BinRelation::WE => {
                let (patten, value) = if self.first.contains("*") || self.first.contains("?") {
                    (WildMatch::new(self.first.as_str()), self.second.as_str())
                } else {
                    (WildMatch::new(self.second.as_str()), self.first.as_str())
                };
                return Ok(patten.matches(value));
            }
        })
    }
}

impl Evaluation for CmpExpress<u32, u32> {
    fn decide(&self, _ctx: ExecContext, _vars_dict: &VarSpace) -> DecideResult {
        Ok(match self.relation {
            BinRelation::EQ => self.first == self.second,
            BinRelation::WE => self.first == self.second,
            BinRelation::NE => self.first != self.second,
            BinRelation::GE => self.first > self.second,
            BinRelation::GT => self.first >= self.second,
            BinRelation::LE => self.first < self.second,
            BinRelation::LT => self.first <= self.second,
        })
    }
}

impl<T, E> Evaluation for CmpExpress<T, E>
where
    T: ValueEval<E> + Debug + Clone,
    E: PartialEq + PartialOrd + WildEq,
    //    A: EvalArgs,
{
    fn decide(&self, _ctx: ExecContext, vars_dict: &VarSpace) -> DecideResult {
        let first = self
            .first
            .eval(vars_dict)
            .map_err(|e| EvalError::ValueError(format!("{:?} , e:{}", self.first.clone(), e)))?;
        match self.relation {
            BinRelation::EQ => Ok(first.eq(&self.second)),
            BinRelation::WE => Ok(first.we(&self.second)),
            BinRelation::NE => Ok(!first.eq(&self.second)),
            BinRelation::GE => Ok(first.ge(&self.second)),
            BinRelation::GT => Ok(first.gt(&self.second)),
            BinRelation::LE => Ok(first.le(&self.second)),
            BinRelation::LT => Ok(first.lt(&self.second)),
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
        let bin_express = CmpExpress::eq("a", "b");
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
        let bin_express = CmpExpress::eq("a", "a");
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
    }
    //test for i32 test bin express
    #[test]
    fn test_bin_express_i32() {
        let bin_express = CmpExpress::eq(1, 2);
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
        let bin_express = CmpExpress::eq(1, 1);
        assert_eq!(bin_express.relation, BinRelation::EQ);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
        let bin_express = CmpExpress::gt(2, 1);
        assert_eq!(bin_express.relation, BinRelation::GT);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(true)
        );
        let bin_express = CmpExpress::gt(1, 2);
        assert_eq!(bin_express.relation, BinRelation::GT);
        assert_eq!(
            bin_express.decide(ExecContext::default(), &VarSpace::default()),
            Ok(false)
        );
    }
}
