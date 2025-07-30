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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_logic_and() {
        let mut input = "&&";
        let result = symbol_logic_and(&mut input);
        assert_eq!(result, Ok(LogicSymbol::And));
        assert_eq!(input, "");

        let mut input = " &&";
        let result = symbol_logic_and(&mut input);
        assert_eq!(result, Ok(LogicSymbol::And));
        assert_eq!(input, "");

        let mut input = "&& test";
        let result = symbol_logic_and(&mut input);
        assert_eq!(result, Ok(LogicSymbol::And));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_logic_or() {
        let mut input = "||";
        let result = symbol_logic_or(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Or));
        assert_eq!(input, "");

        let mut input = " ||";
        let result = symbol_logic_or(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Or));
        assert_eq!(input, "");

        let mut input = "|| test";
        let result = symbol_logic_or(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Or));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_logic_not() {
        let mut input = "!";
        let result = symbol_logic_not(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Not));
        assert_eq!(input, "");

        let mut input = " !";
        let result = symbol_logic_not(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Not));
        assert_eq!(input, "");

        let mut input = "! test";
        let result = symbol_logic_not(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Not));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_match_to() {
        let mut input = "=>";
        let result = symbol_match_to(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " =>";
        let result = symbol_match_to(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "=> test";
        let result = symbol_match_to(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_var() {
        let mut input = "var";
        let result = symbol_var(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " var";
        let result = symbol_var(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "var test";
        let result = symbol_var(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_comma() {
        let mut input = ",";
        let result = symbol_comma(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " ,";
        let result = symbol_comma(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = ", test";
        let result = symbol_comma(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_bracket_end() {
        let mut input = ")";
        let result = symbol_bracket_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " )";
        let result = symbol_bracket_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = ") test";
        let result = symbol_bracket_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_bracket_beg() {
        let mut input = "(";
        let result = symbol_bracket_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " (";
        let result = symbol_bracket_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "( test";
        let result = symbol_bracket_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_brace_end() {
        let mut input = "}";
        let result = symbol_brace_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " }";
        let result = symbol_brace_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "} test";
        let result = symbol_brace_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_brace_beg() {
        let mut input = "{";
        let result = symbol_brace_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " {";
        let result = symbol_brace_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "{ test";
        let result = symbol_brace_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_under_line() {
        let mut input = "_";
        let result = symbol_under_line(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " _";
        let result = symbol_under_line(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "_ test";
        let result = symbol_under_line(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_marvel() {
        let mut input = "!";
        let result = symbol_marvel(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " !";
        let result = symbol_marvel(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "! test";
        let result = symbol_marvel(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_brackets_beg() {
        let mut input = "[";
        let result = symbol_brackets_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " [";
        let result = symbol_brackets_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "[ test";
        let result = symbol_brackets_beg(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_brackets_end() {
        let mut input = "]";
        let result = symbol_brackets_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " ]";
        let result = symbol_brackets_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "] test";
        let result = symbol_brackets_end(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_colon() {
        let mut input = ":";
        let result = symbol_colon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " :";
        let result = symbol_colon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = ": test";
        let result = symbol_colon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_semicolon() {
        let mut input = ";";
        let result = symbol_semicolon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " ;";
        let result = symbol_semicolon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "; test";
        let result = symbol_semicolon(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_pipe() {
        let mut input = "|";
        let result = symbol_pipe(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " |";
        let result = symbol_pipe(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "| test";
        let result = symbol_pipe(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_assign() {
        let mut input = "=";
        let result = symbol_assign(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " =";
        let result = symbol_assign(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "= test";
        let result = symbol_assign(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_dollar() {
        let mut input = "$";
        let result = symbol_dollar(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = " $";
        let result = symbol_dollar(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, "");

        let mut input = "$ test";
        let result = symbol_dollar(&mut input);
        assert_eq!(result, Ok(()));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_eq() {
        let mut input = "==";
        let result = symbol_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Eq));
        assert_eq!(input, "");

        let mut input = " ==";
        let result = symbol_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Eq));
        assert_eq!(input, "");

        let mut input = "== test";
        let result = symbol_cmp_eq(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Eq));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_we() {
        let mut input = "=*";
        let result = symbol_cmp_we(&mut input);
        assert_eq!(result, Ok(CmpSymbol::We));
        assert_eq!(input, "");

        let mut input = " =*";
        let result = symbol_cmp_we(&mut input);
        assert_eq!(result, Ok(CmpSymbol::We));
        assert_eq!(input, "");

        let mut input = "=* test";
        let result = symbol_cmp_we(&mut input);
        assert_eq!(result, Ok(CmpSymbol::We));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_ne() {
        let mut input = "!=";
        let result = symbol_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ne));
        assert_eq!(input, "");

        let mut input = " !=";
        let result = symbol_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ne));
        assert_eq!(input, "");

        let mut input = "!= test";
        let result = symbol_cmp_ne(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ne));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_ge() {
        let mut input = ">=";
        let result = symbol_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ge));
        assert_eq!(input, "");

        let mut input = " >=";
        let result = symbol_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ge));
        assert_eq!(input, "");

        let mut input = ">= test";
        let result = symbol_cmp_ge(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ge));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_gt() {
        let mut input = ">";
        let result = symbol_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Gt));
        assert_eq!(input, "");

        let mut input = " >";
        let result = symbol_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Gt));
        assert_eq!(input, "");

        let mut input = "> test";
        let result = symbol_cmp_gt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Gt));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_le() {
        let mut input = "<=";
        let result = symbol_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Le));
        assert_eq!(input, "");

        let mut input = " <=";
        let result = symbol_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Le));
        assert_eq!(input, "");

        let mut input = "<= test";
        let result = symbol_cmp_le(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Le));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp_lt() {
        let mut input = "<";
        let result = symbol_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Lt));
        assert_eq!(input, "");

        let mut input = " <";
        let result = symbol_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Lt));
        assert_eq!(input, "");

        let mut input = "< test";
        let result = symbol_cmp_lt(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Lt));
        assert_eq!(input, " test");
    }

    #[test]
    fn test_symbol_cmp() {
        let mut input = "==";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Eq));
        assert_eq!(input, "");

        let mut input = "!=";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ne));
        assert_eq!(input, "");

        let mut input = "=*";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::We));
        assert_eq!(input, "");

        let mut input = ">=";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Ge));
        assert_eq!(input, "");

        let mut input = ">";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Gt));
        assert_eq!(input, "");

        let mut input = "<=";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Le));
        assert_eq!(input, "");

        let mut input = "<";
        let result = symbol_cmp(&mut input);
        assert_eq!(result, Ok(CmpSymbol::Lt));
        assert_eq!(input, "");
    }

    #[test]
    fn test_symbol_logic() {
        let mut input = "&&";
        let result = symbol_logic(&mut input);
        assert_eq!(result, Ok(LogicSymbol::And));
        assert_eq!(input, "");

        let mut input = "||";
        let result = symbol_logic(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Or));
        assert_eq!(input, "");

        let mut input = "!";
        let result = symbol_logic(&mut input);
        assert_eq!(result, Ok(LogicSymbol::Not));
        assert_eq!(input, "");
    }

    #[test]
    fn test_error_cases() {
        // Test various error cases
        let mut input = "invalid";
        let result = symbol_logic_and(&mut input);
        assert!(result.is_err());

        let mut input = "invalid";
        let result = symbol_cmp_eq(&mut input);
        assert!(result.is_err());

        let mut input = "invalid";
        let result = symbol_bracket_beg(&mut input);
        assert!(result.is_err());

        let mut input = "";
        let result = symbol_logic_and(&mut input);
        assert!(result.is_err());

        let mut input = "";
        let result = symbol_cmp_eq(&mut input);
        assert!(result.is_err());
    }
}
