use orion_common::cond::CmpOP;
use orion_common::cond::LogicOP;
use winnow::ascii::{multispace0, Caseless};
use winnow::combinator::alt;
use winnow::error::{StrContext, StrContextValue};
use winnow::token::literal;
use winnow::{Parser, Result};

pub fn symbol_sql_cmp_eq(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("=")
        .context(StrContext::Label("symbol_sql"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Eq)
}
pub fn symbol_sql_cmp_ne(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("!=")
        .context(StrContext::Label("symbol_sql"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '!='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Ne)
}
pub fn symbol_sql_cmp_ge(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal(">=")
        .context(StrContext::Label("symbol_sql ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Ge)
}

pub fn symbol_sql_cmp_gt(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal(">")
        .context(StrContext::Label("symbol_sql gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>'",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Gt)
}

pub fn symbol_sql_cmp_le(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("<=")
        .context(StrContext::Label("symbol_sql ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Le)
}

pub fn symbol_sql_cmp_lt(data: &mut &str) -> Result<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("<")
        .context(StrContext::Label("symbol_sql gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<'",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Lt)
}
pub fn symbol_sql_cmp(data: &mut &str) -> Result<CmpOP> {
    alt((
        symbol_sql_cmp_eq,
        symbol_sql_cmp_ne,
        symbol_sql_cmp_le,
        symbol_sql_cmp_ge,
        symbol_sql_cmp_lt,
        symbol_sql_cmp_gt,
    ))
    .parse_next(data)
}

#[derive(Debug, PartialEq, Clone)]
pub enum SQLogicSymbol {
    And,
    Or,
    Not,
}

impl From<SQLogicSymbol> for LogicOP {
    fn from(value: SQLogicSymbol) -> Self {
        match value {
            SQLogicSymbol::And => LogicOP::And,
            SQLogicSymbol::Or => LogicOP::Or,
            SQLogicSymbol::Not => LogicOP::Not,
        }
    }
}

pub fn symbol_sql_logic_and(data: &mut &str) -> Result<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("and"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'and'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::And)
}
pub fn symbol_sql_logic_or(data: &mut &str) -> Result<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("or"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'or'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::Or)
}
pub fn symbol_sql_logic_not(data: &mut &str) -> Result<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("not"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'not'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::Not)
}

pub fn symbol_sql_logic(data: &mut &str) -> Result<SQLogicSymbol> {
    alt((
        symbol_sql_logic_and,
        symbol_sql_logic_or,
        symbol_sql_logic_not,
    ))
    .parse_next(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use orion_common::cond::CmpOP;
    use orion_common::cond::LogicOP;

    #[test]
    fn test_symbol_sql_cmp_eq() {
        let mut input = " = test";
        let result = symbol_sql_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpOP::Eq));
        assert_eq!(input, " test");

        let mut input = "=test";
        let result = symbol_sql_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpOP::Eq));
        assert_eq!(input, "test");

        let mut input = "  =  test";
        let result = symbol_sql_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpOP::Eq));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp_ne() {
        let mut input = " != test";
        let result = symbol_sql_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpOP::Ne));
        assert_eq!(input, " test");

        let mut input = "!=test";
        let result = symbol_sql_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpOP::Ne));
        assert_eq!(input, "test");

        let mut input = "  !=  test";
        let result = symbol_sql_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpOP::Ne));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp_ge() {
        let mut input = " >= test";
        let result = symbol_sql_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpOP::Ge));
        assert_eq!(input, " test");

        let mut input = ">=test";
        let result = symbol_sql_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpOP::Ge));
        assert_eq!(input, "test");

        let mut input = "  >=  test";
        let result = symbol_sql_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpOP::Ge));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp_gt() {
        let mut input = " > test";
        let result = symbol_sql_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpOP::Gt));
        assert_eq!(input, " test");

        let mut input = ">test";
        let result = symbol_sql_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpOP::Gt));
        assert_eq!(input, "test");

        let mut input = "  >  test";
        let result = symbol_sql_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpOP::Gt));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp_le() {
        let mut input = " <= test";
        let result = symbol_sql_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpOP::Le));
        assert_eq!(input, " test");

        let mut input = "<=test";
        let result = symbol_sql_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpOP::Le));
        assert_eq!(input, "test");

        let mut input = "  <=  test";
        let result = symbol_sql_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpOP::Le));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp_lt() {
        let mut input = " < test";
        let result = symbol_sql_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpOP::Lt));
        assert_eq!(input, " test");

        let mut input = "<test";
        let result = symbol_sql_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpOP::Lt));
        assert_eq!(input, "test");

        let mut input = "  <  test";
        let result = symbol_sql_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpOP::Lt));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_cmp() {
        let mut input = " = test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Eq));
        assert_eq!(input, " test");

        let mut input = " != test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Ne));
        assert_eq!(input, " test");

        let mut input = " >= test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Ge));
        assert_eq!(input, " test");

        let mut input = " > test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Gt));
        assert_eq!(input, " test");

        let mut input = " <= test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Le));
        assert_eq!(input, " test");

        let mut input = " < test";
        let result = symbol_sql_cmp(&mut input);
        assert_eq!(result, Ok(CmpOP::Lt));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_sql_logic_and() {
        let mut input = " and test";
        let result = symbol_sql_logic_and(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::And));
        assert_eq!(input, " test");

        let mut input = "AND test";
        let result = symbol_sql_logic_and(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::And));
        assert_eq!(input, " test");

        let mut input = "  and  test";
        let result = symbol_sql_logic_and(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::And));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_logic_or() {
        let mut input = " or test";
        let result = symbol_sql_logic_or(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Or));
        assert_eq!(input, " test");

        let mut input = "OR test";
        let result = symbol_sql_logic_or(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Or));
        assert_eq!(input, " test");

        let mut input = "  or  test";
        let result = symbol_sql_logic_or(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Or));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_logic_not() {
        let mut input = " not test";
        let result = symbol_sql_logic_not(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Not));
        assert_eq!(input, " test");

        let mut input = "NOT test";
        let result = symbol_sql_logic_not(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Not));
        assert_eq!(input, " test");

        let mut input = "  not  test";
        let result = symbol_sql_logic_not(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Not));
        assert_eq!(input, "  test");
    }

    #[test]
    fn test_symbol_sql_logic() {
        let mut input = " and test";
        let result = symbol_sql_logic(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::And));
        assert_eq!(input, " test");

        let mut input = " or test";
        let result = symbol_sql_logic(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Or));
        assert_eq!(input, " test");

        let mut input = " not test";
        let result = symbol_sql_logic(&mut input);
        assert_eq!(result, Ok(SQLogicSymbol::Not));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_sq_logic_symbol_from() {
        let and_symbol = SQLogicSymbol::And;
        let logic_op: LogicOP = and_symbol.into();
        assert_eq!(logic_op, LogicOP::And);

        let or_symbol = SQLogicSymbol::Or;
        let logic_op: LogicOP = or_symbol.into();
        assert_eq!(logic_op, LogicOP::Or);

        let not_symbol = SQLogicSymbol::Not;
        let logic_op: LogicOP = not_symbol.into();
        assert_eq!(logic_op, LogicOP::Not);
    }

    #[test]
    fn test_error_cases() {
        // 测试无效输入
        let mut input = "invalid";
        let result = symbol_sql_cmp_eq(&mut input);
        assert!(result.is_err());

        let mut input = "invalid";
        let result = symbol_sql_logic_and(&mut input);
        assert!(result.is_err());

        let mut input = "";
        let result = symbol_sql_cmp(&mut input);
        assert!(result.is_err());

        let mut input = "";
        let result = symbol_sql_logic(&mut input);
        assert!(result.is_err());
    }
}
