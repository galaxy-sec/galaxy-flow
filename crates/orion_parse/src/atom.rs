use crate::symbol::{symbol_colon, wn_desc};
use winnow::ascii::{multispace0, multispace1, newline, take_escaped, till_line_ending};
use winnow::combinator::{cut_err, delimited, repeat};
use winnow::error::{ContextError, ErrMode, StrContext, StrContextValue};
use winnow::token::{literal, one_of, take_till, take_until, take_while};
use winnow::{ModalResult, Parser};

pub fn take_var_name(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_'])).parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_var_path(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_json_path(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(
        1..,
        ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.', '/', '[', ']']),
    )
    .parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_wild_key(input: &mut &str) -> ModalResult<String> {
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
pub fn take_path(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.', '/'], '-'))
        .parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_obj_path(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '/'])).parse_next(input)?;
    let _ = multispace1.parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_obj_wild_path(input: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(input)?;
    let key =
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '/', '*'])).parse_next(input)?;
    let _ = multispace1.parse_next(input)?;
    Ok(key.to_string())
}

pub fn take_key_pair(input: &mut &str) -> ModalResult<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    symbol_colon.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let val = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}

pub fn take_key_val(input: &mut &str) -> ModalResult<(String, String)> {
    let _ = multispace0.parse_next(input)?;
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '.'])).parse_next(input)?;
    symbol_colon.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let val = take_till(1.., |c| c == ',' || c == ';').parse_next(input)?;
    Ok((key.to_string(), val.to_string()))
}
pub fn take_empty(input: &mut &str) -> ModalResult<()> {
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

pub fn take_parentheses_val(data: &mut &str) -> ModalResult<String> {
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

fn inner_parentheses_val(data: &mut &str, depth: &mut u32) -> ModalResult<String> {
    let mut target_val = take_till(0.., |c| c == '(' || c == ')')
        .parse_next(data)?
        .to_string();

    match literal::<&str, &str, winnow::error::InputError<&str>>("(").parse_next(data) {
        Ok(_) => {
            *depth += 1;
            let val = inner_parentheses_val(data, depth)?;
            target_val.push_str(&format!("({}", val));
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

pub fn take_parentheses_scope(data: &mut &str) -> ModalResult<(String, String)> {
    let _ = multispace0.parse_next(data)?;
    literal("(").parse_next(data)?;
    let beg = take_till(0.., |x| x == ',').parse_next(data)?;
    literal(",").parse_next(data)?;
    let _ = multispace0.parse_next(data)?;
    let end = take_till(0.., |x| x == ')').parse_next(data)?;
    literal(")").parse_next(data)?;
    Ok((beg.into(), end.into()))
}

//take string
//eg:  "hello", "a/b/c", "\""
pub fn take_string(data: &mut &str) -> ModalResult<String> {
    // 使用 take_escaped 解析转义字符
    let string_parser = take_escaped(
        take_while(1.., |c: char| c != '"' && c != '\\'), // 普通字符的条件
        '\\',                                             // 转义字符
        one_of(['"', 'n', '\\']),                         // 可转义的字符（包括 /）
    );

    // 使用 delimited 解析双引号包围的字符串，并将结果转换为 String
    //preceded(
    delimited(
        '"',
        cut_err(string_parser.map(String::from)), // 将 &str 转换为 String
        '"',
    )
    .context(StrContext::Label("string"))
    .parse_next(data)
}

//take var name.
// eg : ${name}  -> name
pub fn take_env_var(data: &mut &str) -> ModalResult<String> {
    let _ = multispace0.parse_next(data)?;
    delimited(
        "${",
        cut_err(
            take_till(1.., |c| c == '}')
                .map(|s: &str| s.trim())
                .verify(|s: &str| !s.is_empty())
                .context(StrContext::Expected(StrContextValue::Description(
                    "non-empty variable name",
                ))),
        ),
        cut_err("}").context(StrContext::Expected(StrContextValue::Description(
            "missing closing '}'",
        ))),
    )
    .context(StrContext::Label("environment variable"))
    .parse_next(data)
    .map(|s: &str| s.to_string())
}
//take raw sting by ^""^
//eg:  ^"hello"^ , ^"hell"0"^
pub fn gal_raw_string(data: &mut &str) -> ModalResult<String> {
    delimited(
        "r#\"",
        take_until(0.., "\"#"),
        "\"#".context(wn_desc("<raw-end>")),
    )
    .context(StrContext::Label("<raw string>"))
    .parse_next(data)
    .map(String::from)
}

pub fn skip_spaces_block(input: &mut &str) -> ModalResult<()> {
    let _: Vec<()> = repeat(0.., skip_spaces_line).parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

pub fn skip_spaces_line(input: &mut &str) -> ModalResult<()> {
    let _ = multispace0.parse_next(input)?;
    //let _ = alt((newline.map(|_| ()), multispace1.map(|_| ()))).parse_next(input)?;
    let _ = newline.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    Ok(())
}

/*
pub fn starts_with(literal_str: &str, input: &str) -> bool {
    literal::<&str, &str, ErrMode<ContextError>>(literal_str)
        .parse_peek(input)
        .is_ok()
}
*/

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
    use crate::atom::{
        gal_raw_string, skip_spaces_block, take_env_var, take_string, take_var_name,
    };
    use winnow::{ModalResult, Parser};

    #[test]
    fn test_key_val() -> ModalResult<()> {
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
    fn test_take_string() {
        // 测试普通字符串
        let mut input = r#""hello""#;
        assert_eq!(take_string(&mut input), Ok("hello".to_string()));

        // 测试包含转义字符的字符串
        //let mut input = r#""a\/b\/c""#;
        //assert_eq!(take_string(&mut input), Ok("a/b/c".to_string()));

        // 测试包含转义双引号的字符串
        let mut input = r#""M\"hello\"""#;
        let t_out = take_string(&mut input);
        println!("{}", input);
        assert_eq!(t_out, Ok(r#"M\"hello\""#.to_string()));

        // 测试空字符串
        let mut input = r#""""#;
        assert_eq!(take_string(&mut input), Ok("".to_string()));

        // 测试无效输入（缺少结尾双引号）
        let mut input = r#""hello"#;
        assert!(take_string(&mut input).is_err());

        // 测试无效输入（未转义的双引号）
        let mut input = r#""hello"world""#;
        assert_eq!(take_string(&mut input), Ok("hello".to_string()));
    }

    #[test]
    fn test_gal_raw_string() {
        // 测试普通原始字符串
        let mut input = "r#\"hello\"#";
        assert_eq!(gal_raw_string(&mut input), Ok("hello".to_string()));
        println!("{}", input);

        // 测试包含特殊字符的原始字符串
        let mut input = "r#\"hell\\\"0\"#";
        let t_out = gal_raw_string(&mut input);
        println!("{}", input);
        assert_eq!(t_out, Ok(r#"hell\"0"#.to_string()));

        // 测试空字符串
        let mut input = "r#\"\"#";
        assert_eq!(gal_raw_string(&mut input), Ok("".to_string()));

        // 测试无效输入（缺少结尾标记）
        let mut input = r#"r#"hello"#;
        assert!(gal_raw_string(&mut input).is_err());
        // 测试无效输入（缺少开头标记）
        let mut input = r#""hello"\#"#;
        assert!(gal_raw_string(&mut input).is_err());

        let mut input =
            "r#\"{\"branchs\" : [{ \"name\": \"develop\" }, { \"name\" : \"release/1\"}]} \"#;";
        assert!(gal_raw_string(&mut input).is_ok());
    }

    #[test]
    fn valid_variable() {
        let mut input = "${name}";
        assert_eq!(take_env_var(&mut input), Ok("name".to_string()));
        assert_eq!(input, "");
    }

    #[test]
    fn trailing_characters() {
        let mut input = "${var}remaining";
        assert_eq!(take_env_var(&mut input), Ok("var".to_string()));
        assert_eq!(input, "remaining");
    }

    #[test]
    fn missing_closing_brace() {
        let mut input = "${var";
        assert!(take_env_var(&mut input).is_err());
    }

    #[test]
    fn no_opening_brace() {
        let mut input = "var}";
        assert!(take_env_var(&mut input).is_err());
    }

    #[test]
    fn empty_variable() {
        let mut input = "${}";
        assert!(take_env_var(&mut input).is_err());
    }

    #[test]
    fn special_characters_in_name() {
        let mut input = "${var-name_123}";
        assert_eq!(take_env_var(&mut input), Ok("var-name_123".to_string()));
        assert_eq!(input, "");
    }

    #[test]
    fn nested_braces_in_name() {
        let mut input = "${var{abc}}";
        assert_eq!(take_env_var(&mut input), Ok("var{abc".to_string()));
        assert_eq!(input, "}");
    }

    #[test]
    fn trimmed_spaces() {
        let mut input = "${  spaced_var  }";
        assert_eq!(take_env_var(&mut input), Ok("spaced_var".to_string()));
        assert_eq!(input, "");
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
    fn test_ignore_spaces_block_empty() -> ModalResult<()> {
        let mut input = "";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_only_spaces() -> ModalResult<()> {
        let mut input = "   ";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_only_newlines() -> ModalResult<()> {
        let mut input = "\n\n\n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_mixed_spaces_and_newlines() -> ModalResult<()> {
        let mut input = "  \n  \n  \n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_trailing_text() -> ModalResult<()> {
        let mut input = "  \n  \n  \n  some text";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_no_spaces() -> ModalResult<()> {
        let mut input = "some text";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text");
        Ok(())
    }

    #[test]
    fn test_ignore_spaces_block_mixed_spaces_newlines_and_text() -> ModalResult<()> {
        let mut input = "  \n  some text\n  more text  \n";
        skip_spaces_block(&mut input)?;
        assert_eq!(input, "some text\n  more text  \n");
        Ok(())
    }
}
