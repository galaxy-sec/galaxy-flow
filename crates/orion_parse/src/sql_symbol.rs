use orion_common::cond::CmpOP;
use orion_common::cond::LogicOP;
use winnow::ascii::{multispace0, Caseless};
use winnow::combinator::alt;
use winnow::error::{StrContext, StrContextValue};
use winnow::token::literal;
use winnow::{ModalResult, Parser};

pub fn symbol_sql_cmp_eq(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("=")
        .context(StrContext::Label("symbol_sql"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Eq)
}
pub fn symbol_sql_cmp_ne(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("!=")
        .context(StrContext::Label("symbol_sql"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '!='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Ne)
}
pub fn symbol_sql_cmp_ge(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal(">=")
        .context(StrContext::Label("symbol_sql ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Ge)
}

pub fn symbol_sql_cmp_gt(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal(">")
        .context(StrContext::Label("symbol_sql gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>'",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Gt)
}

pub fn symbol_sql_cmp_le(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("<=")
        .context(StrContext::Label("symbol_sql ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<='",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Le)
}

pub fn symbol_sql_cmp_lt(data: &mut &str) -> ModalResult<CmpOP> {
    let _ = multispace0.parse_next(data)?;
    literal("<")
        .context(StrContext::Label("symbol_sql gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<'",
        )))
        .parse_next(data)?;
    Ok(CmpOP::Lt)
}
pub fn symbol_sql_cmp(data: &mut &str) -> ModalResult<CmpOP> {
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

pub fn symbol_sql_logic_and(data: &mut &str) -> ModalResult<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("and"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'and'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::And)
}
pub fn symbol_sql_logic_or(data: &mut &str) -> ModalResult<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("or"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'or'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::Or)
}
pub fn symbol_sql_logic_not(data: &mut &str) -> ModalResult<SQLogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(Caseless("not"))
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'not'",
        )))
        .parse_next(data)?;
    Ok(SQLogicSymbol::Not)
}

pub fn symbol_sql_logic(data: &mut &str) -> ModalResult<SQLogicSymbol> {
    alt((
        symbol_sql_logic_and,
        symbol_sql_logic_or,
        symbol_sql_logic_not,
    ))
    .parse_next(data)
}
