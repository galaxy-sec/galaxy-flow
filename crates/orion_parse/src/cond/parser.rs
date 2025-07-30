use std::collections::HashMap;

use crate::atom::take_var_name;
use crate::cond::{CmpParser, SymbolFrom, WnCondParser};
use crate::symbol::{symbol_bracket_beg, symbol_cmp, symbol_dollar, CmpSymbol, LogicSymbol};
use orion_common::cond::{
    cmp_is_true, CmpOP, CmpSymbolDef, CompareExpress, Condition, ExpressEnum, LogicCrator,
    LogicExpress, LogicOP, LogicSymbolDef, RustSymbol, SQLSymbol, ValueGet,
};
use winnow::ascii::{digit1, multispace0};
use winnow::combinator::peek;
use winnow::error::{ParserError, StrContext, StrContextValue};
use winnow::token::literal;
use winnow::{Parser, Result};

use super::LogicSymbolGet;

impl SymbolFrom<LogicSymbol> for LogicOP {
    fn op_from(value: LogicSymbol) -> Self {
        match value {
            LogicSymbol::And => LogicOP::And,
            LogicSymbol::Or => LogicOP::Or,
            LogicSymbol::Not => LogicOP::Not,
        }
    }
}

impl SymbolFrom<CmpSymbol> for CmpOP {
    fn op_from(value: CmpSymbol) -> Self {
        match value {
            CmpSymbol::We => CmpOP::We,
            CmpSymbol::Eq => CmpOP::Eq,
            CmpSymbol::Ne => CmpOP::Ne,
            CmpSymbol::Gt => CmpOP::Gt,
            CmpSymbol::Ge => CmpOP::Ge,
            CmpSymbol::Lt => CmpOP::Lt,
            CmpSymbol::Le => CmpOP::Le,
        }
    }
}
impl<T, H, S> WnCondParser<T, H, S>
where
    H: CmpParser<T, S>,
    S: LogicSymbolGet + LogicSymbolDef + CmpSymbolDef,
{
    pub fn lev2_exp(data: &mut &str, stop: Option<&str>) -> Result<ExpressEnum<T, S>> {
        let mut left: Option<ExpressEnum<T, S>> = None;
        loop {
            multispace0.parse_next(data)?;
            if data.is_empty() {
                break;
            }
            if let Some(stop) = stop {
                if peek_str(stop, data).is_ok() {
                    literal(stop).parse_next(data)?;
                    break;
                }
            }
            if peek_str("(", data).is_ok() {
                let group = Self::group_exp.parse_next(data)?;
                left = Some(group);
                continue;
            } else if peek_str(S::symbol_not(), data).is_ok() {
                S::logic_not.parse_next(data)?;
                let right = Self::lev0_exp(data, stop)?;
                left = Some(ExpressEnum::from_not(right));
                continue;
            } else if peek_str(S::symbol_and(), data).is_ok() {
                S::logic_and.parse_next(data)?;
                let right = Self::lev1_exp(data, stop)?;
                left = Some(ExpressEnum::Logic(LogicExpress::new(
                    LogicOP::And,
                    left,
                    right,
                )));
                continue;
            } else if peek_str(S::symbol_or(), data).is_ok() {
                S::logic_or.parse_next(data)?;
                let right = Self::lev2_exp(data, stop)?;
                left = Some(ExpressEnum::Logic(LogicExpress::new(
                    LogicOP::Or,
                    left,
                    right,
                )));
                continue;
            } else {
                let compare = H::cmp_exp.parse_next(data)?;
                left = Some(ExpressEnum::Compare(compare));
                continue;
            }
        }
        match left {
            Some(o) => Ok(o),
            None => Err(ParserError::from_input(data)),
        }
    }

    #[allow(clippy::never_loop)]
    fn lev0_exp(data: &mut &str, stop: Option<&str>) -> Result<ExpressEnum<T, S>> {
        let mut left: Option<ExpressEnum<T, S>> = None;
        loop {
            multispace0.parse_next(data)?;
            if data.is_empty() {
                break;
            }
            if let Some(stop) = stop {
                if peek_str(stop, data).is_ok() {
                    literal(stop).parse_next(data)?;
                    break;
                }
            }
            //only one segment;
            if peek_str("(", data).is_ok() {
                let group = Self::group_exp.parse_next(data)?;
                left = Some(group);
                break;
            } else {
                let compare = H::cmp_exp.parse_next(data)?;
                left = Some(ExpressEnum::Compare(compare));
                break;
            }
        }
        match left {
            Some(o) => Ok(o),
            None => Err(ParserError::from_input(
                data, //&"overall express data not empty",
                     //ErrorKind::Token,
            )),
        }
    }

    fn lev1_exp(data: &mut &str, stop: Option<&str>) -> Result<ExpressEnum<T, S>> {
        let mut left: Option<ExpressEnum<T, S>> = None;
        loop {
            multispace0.parse_next(data)?;
            if data.is_empty() {
                break;
            }
            if let Some(stop) = stop {
                if peek_str(stop, data).is_ok() {
                    literal(stop).parse_next(data)?;
                    break;
                }
            }
            if peek_str("(", data).is_ok() {
                let group = Self::group_exp.parse_next(data)?;
                left = Some(group);
                continue;
            } else if peek_str(S::symbol_not(), data).is_ok() {
                S::logic_not.parse_next(data)?;
                let right = Self::lev0_exp(data, stop)?;
                left = Some(ExpressEnum::from_not(right));
                continue;
            } else if peek_str(S::symbol_and(), data).is_ok() {
                S::logic_and.parse_next(data)?;
                let right = Self::lev1_exp(data, stop)?;
                left = Some(ExpressEnum::Logic(LogicExpress::new(
                    LogicOP::And,
                    left,
                    right,
                )));
                continue;
            } else if peek_str("||", data).is_ok() {
                break;
            } else {
                let compare = H::cmp_exp.parse_next(data)?;
                left = Some(ExpressEnum::Compare(compare));
                continue;
            }
        }
        match left {
            Some(o) => Ok(o),
            None => Err(ParserError::from_input(data)),
        }
    }

    fn group_exp(data: &mut &str) -> Result<ExpressEnum<T, S>> {
        multispace0.parse_next(data)?;
        symbol_bracket_beg.parse_next(data)?;
        Self::lev2_exp(data, Some(")"))
    }
}
fn peek_str(what: &str, input: &mut &str) -> Result<()> {
    peek(what).parse_next(input)?;
    Ok(())
}

impl LogicSymbolGet for RustSymbol {
    fn logic_and(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("&&")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need '&&'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::And)
    }

    fn logic_or(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("||")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need '||'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::Or)
    }

    fn logic_not(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("!")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need '!'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::Not)
    }
}

impl LogicSymbolGet for SQLSymbol {
    fn logic_and(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("and")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need 'and'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::And)
    }

    fn logic_or(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("or")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need 'or'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::Or)
    }

    fn logic_not(data: &mut &str) -> Result<LogicSymbol> {
        let _ = multispace0.parse_next(data)?;
        literal("not")
            .context(StrContext::Label("symbol"))
            .context(StrContext::Expected(StrContextValue::Description(
                "need 'not'",
            )))
            .parse_next(data)?;
        Ok(LogicSymbol::Not)
    }
}

pub struct ObjGet {}
impl CmpParser<u32, RustSymbol> for ObjGet {
    fn cmp_exp(data: &mut &str) -> Result<CompareExpress<u32, RustSymbol>> {
        symbol_dollar.parse_next(data)?;
        let var_name = take_var_name(data)?;
        let op = symbol_cmp.parse_next(data)?;
        multispace0.parse_next(data)?;
        let target = digit1.parse_next(data)?;
        let ins = CompareExpress::new(
            CmpOP::op_from(op),
            var_name.to_string(),
            target.parse::<u32>().unwrap(),
        );
        Ok(ins)
    }
}

struct VMap(HashMap<&'static str, u32>);
impl ValueGet<u32> for VMap {
    fn value_get(&self, var: &str) -> Option<&u32> {
        self.0.get(var)
    }
}

impl Condition<VMap> for LogicExpress<u32, RustSymbol> {
    fn is_true(&self, data: &VMap) -> bool {
        cmp_is_true(&self.op, self.left.as_ref(), &self.right, data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    type SVMap = HashMap<&'static str, u32>;
    type CondParser = WnCondParser<u32, ObjGet, RustSymbol>;
    #[test]
    pub fn test_express_exec_simple() -> Result<()> {
        let data = SVMap::from([("A", 100), ("B", 200)]);

        let mut code = r#"$A == 100"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A =* 100"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A >= 100"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A <= 100"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A != 100"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(!exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A > 90 && $B > 150"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A > 100 && $B > 150"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(!exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A > 100 || $B > 150"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        let mut code = r#"$A < 10 || ($A >= 100 && $B > 150)"#;
        let exp = CondParser::exp(&mut code)?;
        assert!(exp.is_true(&VMap(data.clone())));

        Ok(())
    }
}

/*
 */
