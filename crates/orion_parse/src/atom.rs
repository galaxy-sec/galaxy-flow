use crate::symbol::symbol_colon;
use winnow::ascii::{multispace0, multispace1, newline, till_line_ending};
use winnow::combinator::repeat;
use winnow::error::{ContextError, ErrMode, StrContext, StrContextValue};
use winnow::token::{literal, take_till, take_while};
use winnow::{Parser, Result};

pub fn take_var_name(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_'])).parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_var_full_name(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.', '[', ']']))
        .parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_var_path(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_json_path(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(
        1..,
        ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.', '/', '[', ']']),
    )
    .parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_wild_key(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(
        1..,
        (
            '0'..='9',
            'A'..='Z',
            'a'..='z',
            ['_', '.', '*', '/', '[', ']'],
        ),
    )
    .parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_path(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.', '/'], '-'))
        .parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_obj_path(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '/'])).parse_next(input)?;
    let _ = multispace1.parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_obj_wild_path(input: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(input)?;
    let key =
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '/', '*'])).parse_next(input)?;
    let _ = multispace1.parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_key_pair(input: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    symbol_colon.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let val = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}

pub fn take_key_val(input: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    symbol_colon.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let val = take_till(1.., |c| c == ',' || c == ';').parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}
pub fn take_empty(input: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

pub fn take_parentheses_val(data: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(data)?;
    literal("(")
        .context(StrContext::Label("syntax"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need match '(x)', lack '('",
        )))
        .parse_next(data)?;
    let target_val = inner_parentheses_val(data, &mut 0)?;
    literal(")")
        .context(StrContext::Label("syntax"))
        .context(StrContext::Expected(StrContextValue::Description(
            "need match '(x)', lack ')'",
        )))
        .parse_next(data)?;
    Ok(target_val.trim().to_string())
}

fn inner_parentheses_val(data: &mut &str, depth: &mut u32) -> Result<String> {
    let mut target_val = take_till(0.., |c| c == '(' || c == ')')
        .parse_next(data)?
        .to_string();

    match literal::<&str, &str, winnow::error::InputError<&str>>("(").parse_next(data) {
        Ok(_) => {
            *depth += 1;
            let val = inner_parentheses_val(data, depth)?;
            target_val.push_str(&format!("({val}"));
        }
        Err(_) => {
            if *depth != 0 {
                literal(")").parse_next(data)?;
                target_val.push(')');
            }
        }
    }
    Ok(target_val)
}

pub fn take_parentheses_scope(data: &mut &str) -> Result<(String, String)> {
    let _ = multispace0.parse_next(data)?;
    literal("(").parse_next(data)?;
    let beg = take_till(0.., |x| x == ',').parse_next(data)?;
    literal(",").parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let end = take_till(0.., |x| x == ')').parse_next(data)?;
    literal(")").parse_next(data)?;
    Ok((beg.into(), end.into()))
}

pub fn skip_spaces_block(input: &mut &str) -> Result<()> {
    let _: Vec<()> = repeat(0.., skip_spaces_line).parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

pub fn skip_spaces_line(input: &mut &str) -> Result<()> {
    let _ = multispace0.parse_next(input)?;
    //let _ = alt((newline.map(|_| ()), multispace1.map(|_| ()))).parse_next(input)?;
    let _ = newline.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

pub fn starts_with<'a, F, O>(mut parser: F, input: &'a str) -> bool
where
    F: Parser<&'a str, O, ErrMode<ContextError>>,
{
    parser.parse_peek(input).is_ok()
}
pub fn peek_line(input: &str) -> String {
    if let Ok((_, what)) = till_line_ending::<&str, ErrMode<ContextError>>.parse_peek(input) {
        return what.to_string();
    }
    "".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use winnow::{Parser, Result};

    #[test]
    fn test_key_val() -> Result<()> {
        let mut data = "x";
        let key = take_var_name.parse_next(&mut data)?;
        assert_eq!(key, "x");

        let mut data = "x(10)";
        let key = take_var_name.parse_next(&mut data)?;
        assert_eq!(key, "x");

        let mut data = "x10(10)";
        let key = take_var_name.parse_next(&mut data)?;
        assert_eq!(key, "x10");

        let mut data = " x10 (10)";
        let key = take_var_name.parse_next(&mut data)?;
        assert_eq!(key, "x10");

        let mut data = " x_1 (10)";
        let key = take_var_name.parse_next(&mut data)?;
        assert_eq!(key, "x_1");

        Ok(())
    }

    #[test]
    fn test_starts_with_parser() {
        use super::*;
        use winnow::token::literal;

        // 测试字面量匹配
        assert!(starts_with(literal("hello"), "hello world"));
        assert!(!starts_with(literal("hello"), "world hello"));

        assert!(starts_with((multispace0, literal("hello")), "hello world"));
    }

    #[test]
    fn test_ignore_spaces_block_empty() -> Result<()> {
        let mut input = "";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_only_spaces() -> Result<()> {
        let mut input = "   ";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_only_newlines() -> Result<()> {
        let mut input = "\n\n\n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_mixed_spaces_and_newlines() -> Result<()> {
        let mut input = "  \n  \n  \n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_trailing_text() -> Result<()> {
        let mut input = "  \n  \n  \n  some text";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_no_spaces() -> Result<()> {
        let mut input = "some text";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_mixed_spaces_newlines_and_text() -> Result<()> {
        let mut input = "  \n  some text\n  more text  \n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text\n  more text  \n");
        Ok(())
    }
}
