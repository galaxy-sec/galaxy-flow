use orion_sec::sec::SecValueType;
use wildmatch::WildMatch;

use crate::calculate::compare::CmpExpress;
use crate::calculate::logic::LogicExpress;
use crate::calculate::traits::{DecideResult, Evaluation, WildEq};
use crate::context::ExecContext;
use crate::execution::VarSpace;
use crate::primitive::GxlObject;

use super::defined::BoolBinFn;
use super::dynval::{EnvVarTag, EvalError, EvalResult, ValueEval, VarDef};
use std::fmt::Debug;
use std::num::ParseIntError;

impl From<ParseIntError> for EvalError {
    fn from(_: ParseIntError) -> Self {
        EvalError::ParseError
    }
}
impl WildEq for u32 {
    fn we(&self, other: &Self) -> bool {
        self == other
    }
}

impl WildEq for String {
    fn we(&self, other: &Self) -> bool {
        let (patten, value) = if self.contains("*") || self.contains("?") {
            (WildMatch::new(self.as_str()), other.as_str())
        } else {
            (WildMatch::new(other.as_str()), self.as_str())
        };
        patten.matches(value)
    }
}

pub type EVarDef = VarDef<String, EnvVarTag>;
#[derive(Clone, Debug)]
pub enum ExpressEnum {
    Logic(Box<LogicExpress<ExpressEnum, ExpressEnum>>),
    Cmp(CmpExpress<GxlObject, GxlObject>),
    Fun(BoolBinFn),
}

impl ValueEval<GxlObject> for GxlObject {
    fn eval(&self, vars: &VarSpace) -> EvalResult<GxlObject> {
        match self {
            GxlObject::VarRef(name) => vars
                .get(name)
                .map(GxlObject::from)
                .ok_or_else(|| EvalError::VarMiss(name.clone())),
            GxlObject::Value(_) => Ok(self.clone()),
        }
    }
}
impl WildEq for GxlObject {
    fn we(&self, other: &Self) -> bool {
        match (self, other) {
            (GxlObject::VarRef(f), GxlObject::VarRef(s)) => f.eq(s),
            (GxlObject::Value(f), GxlObject::Value(s)) => match (f, s) {
                (SecValueType::String(s1), SecValueType::String(s2)) => s1.value().we(s2.value()),
                _ => {
                    unreachable!("unsupport we op! only for string ");
                }
            },
            _ => {
                unreachable!("unsupport we op! only for string ");
            }
        }
        //self.eq(other)
    }
}

impl<T> From<T> for ExpressEnum
where
    T: Into<BoolBinFn>,
{
    fn from(value: T) -> Self {
        ExpressEnum::Fun(value.into())
    }
}

impl Evaluation for ExpressEnum {
    fn decide(&self, ctx: ExecContext, args: &VarSpace) -> DecideResult {
        match self {
            ExpressEnum::Cmp(x) => x.decide(ctx, args),
            ExpressEnum::Fun(x) => x.decide(ctx, args),
            ExpressEnum::Logic(x) => x.decide(ctx, args),
        }
    }
}
impl ExpressEnum {}

#[cfg(test)]
mod tests {
    use orion_sec::sec::SecFrom;

    use crate::{calculate::compare::BinRelation, execution::VarSpace};

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

    #[test]
    fn test_gxl_object_we() {
        use crate::primitive::GxlObject;

        // 测试字符串通配符匹配
        let obj1 = GxlObject::Value(SecValueType::nor_from("hello*".to_string()));
        let obj2 = GxlObject::Value(SecValueType::nor_from("hello_world".to_string()));
        assert!(obj1.we(&obj2));

        // 测试字符串不匹配
        let obj3 = GxlObject::Value(SecValueType::nor_from("world*".to_string()));
        assert!(!obj3.we(&obj2));

        // 测试非字符串类型（应触发 unreachable!）
        //let obj4 = GxlObject::Value(SecValueType::nor_from(42));
        //let obj5 = GxlObject::Value(SecValueType::nor_from(42));
        // 注意：这里会触发 unreachable!，因为 we 方法不支持非字符串类型
        // 测试时可以通过 #[should_panic] 注解捕获 panic
        // #[should_panic(expected = "unsupport we op! only for string")]
        // assert!(obj4.we(&obj5));
    }
}
