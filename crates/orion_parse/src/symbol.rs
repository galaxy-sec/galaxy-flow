use winnow::ascii::multispace0;
use winnow::combinator::alt;
use winnow::error::{StrContext, StrContextValue};
use winnow::token::literal;
use winnow::{Parser, Result};

#[derive(Debug, PartialEq, Clone)]
pub enum LogicSymbol {
    And,
    Or,
    Not,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum CmpSymbol {
    // width match =*
    We,
    Eq,
    Ne,
    Gt,
    Ge,
    Lt,
    Le,
}

pub fn symbol_logic_and(data: &mut &str) -> Result<LogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("&&")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '&&'",
        )))
        .parse_next(data)?;
    Ok(LogicSymbol::And)
}
pub fn symbol_logic_or(data: &mut &str) -> Result<LogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("||")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '||'",
        )))
        .parse_next(data)?;
    Ok(LogicSymbol::Or)
}
pub fn symbol_logic_not(data: &mut &str) -> Result<LogicSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("!")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '!'",
        )))
        .parse_next(data)?;
    Ok(LogicSymbol::Not)
}
pub fn symbol_match_to(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("=>")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '=>'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_var(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("var")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need 'var'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_comma(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal(",")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need ','",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_bracket_end(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal(")")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need ')'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_bracket_beg(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("(")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '('",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_brace_end(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("}")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '}'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_brace_beg(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("{")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '{'",
        )))
        .parse_next(data)?;
    Ok(())
}
pub fn symbol_under_line(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("_")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '_'",
        )))
        .parse_next(data)?;
    Ok(())
}
pub fn symbol_marvel(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("!")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '!'",
        )))
        .parse_next(data)?;
    Ok(())
}
//Brackets

pub fn symbol_brackets_beg(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("[")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '['",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_brackets_end(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("]")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need ']'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_colon(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal(":")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need ':'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_semicolon(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal(";")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need ';'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_pipe(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("|")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '|' pipe symbol",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_assign(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("=")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '='",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_dollar(data: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(data)?;
    literal("$")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '$'",
        )))
        .parse_next(data)?;
    Ok(())
}

pub fn symbol_cmp_eq(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("==")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '=='",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Eq)
}
pub fn symbol_cmp_we(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("=*")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '=*'",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::We)
}
pub fn symbol_cmp_ne(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("!=")
        .context(StrContext::Label("symbol"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '!='",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Ne)
}
pub fn symbol_cmp_ge(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(">=")
        .context(StrContext::Label("symbol ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>='",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Ge)
}

pub fn symbol_cmp_gt(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal(">")
        .context(StrContext::Label("symbol gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '>'",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Gt)
}

pub fn symbol_cmp_le(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("<=")
        .context(StrContext::Label("symbol ge"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<='",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Le)
}

pub fn symbol_cmp_lt(data: &mut &str) -> Result<CmpSymbol> {
    let _ = multispace0.parse_next(data)?;
    literal("<")
        .context(StrContext::Label("symbol gt"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need '<'",
        )))
        .parse_next(data)?;
    Ok(CmpSymbol::Lt)
}
pub fn symbol_cmp(data: &mut &str) -> Result<CmpSymbol> {
    alt((
        symbol_cmp_eq,
        symbol_cmp_ne,
        symbol_cmp_we,
        symbol_cmp_le,
        symbol_cmp_ge,
        symbol_cmp_lt,
        symbol_cmp_gt,
    ))
    .parse_next(data)
}

pub fn symbol_logic(data: &mut &str) -> Result<LogicSymbol> {
    alt((symbol_logic_and, symbol_logic_or, symbol_logic_not)).parse_next(data)
}

#[inline(always)]
pub fn wn_label(label: &'static str) -> StrContext {
    StrContext::Label(label)
}

#[inline(always)]
pub fn wn_literal(lit: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::StringLiteral(lit))
}

#[inline(always)]
pub fn wn_desc(desc: &'static str) -> StrContext {
    StrContext::Expected(StrContextValue::Description(desc))
}
