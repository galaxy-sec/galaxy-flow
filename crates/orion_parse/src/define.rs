use crate::symbol::wn_desc;
use winnow::ascii::{digit1, multispace0, take_escaped};
use winnow::combinator::{alt, delimited, fail};
use winnow::error::{StrContext, StrContextValue};
use winnow::token::{literal, one_of, take_till, take_until, take_while};
use winnow::{Parser, Result};
//take string
pub fn take_string(data: &mut &str) -> Result<String> {
    // 使用 take_escaped 解析转义字符
    let string_parser = take_escaped(
        take_while(1.., |c: char| c != '"' && c != '\\'), // 普通字符的条件
        '\\',                                             // 转义字符
        one_of(['"', 'n', '\\']),                         // 可转义的字符（包括 /）
    );

    delimited(
        '"',
        string_parser.map(String::from), // 将 &str 转换为 String
        '"',
    )
    .context(StrContext::Label("string"))
    .parse_next(data)
}

pub fn take_number(data: &mut &str) -> Result<u64> {
    // 使用 take_escaped 解析转义字符
    let digit = digit1
        .context(StrContext::Label("number"))
        .parse_next(data)?;
    if let Ok(x) = digit.parse::<u64>() {
        return Ok(x);
    }
    fail.context(wn_desc("number")).parse_next(data)
}
pub fn take_float(data: &mut &str) -> Result<f64> {
    // 使用 take_escaped 解析转义字符
    let integer_part = digit1
        .context(StrContext::Label("float"))
        .parse_next(data)?;
    literal(".").parse_next(data)?;
    let fractional_part = digit1
        .context(StrContext::Label("float"))
        .parse_next(data)?;
    // 组合整数和小数部分
    let float_str = format!("{integer_part}.{fractional_part}",);
    if let Ok(x) = float_str.parse::<f64>() {
        return Ok(x);
    }
    fail.context(wn_desc("float")).parse_next(data)
}

pub fn take_bool(data: &mut &str) -> Result<bool> {
    alt((
        literal("true").map(|_| true),
        literal("TRUE").map(|_| true),
        literal("false").map(|_| false),
        literal("FALSE").map(|_| false),
    ))
    .parse_next(data)
}

//take var name.
// eg : ${name}  -> name
pub fn take_var_ref_name(data: &mut &str) -> Result<String> {
    let _ = multispace0.parse_next(data)?;
    delimited(
        "${",
        take_till(1.., |c| c == '}')
            .map(|s: &str| s.trim())
            .verify(|s: &str| !s.is_empty())
            .context(StrContext::Expected(StrContextValue::Description(
                "non-empty variable name",
            ))),
        "}".context(StrContext::Expected(StrContextValue::Description(
            "missing closing '}'",
        ))),
    )
    .context(StrContext::Label("environment variable"))
    .parse_next(data)
    .map(|s: &str| s.to_string())
}
//take raw sting by ^""^
//eg:  ^"hello"^ , ^"hell"0"^
pub fn gal_raw_str(data: &mut &str) -> Result<String> {
    delimited(
        "r#\"",
        take_until(0.., "\"#"),
        "\"#".context(wn_desc("<raw-end>")),
    )
    .context(StrContext::Label("<raw string>"))
    .parse_next(data)
    .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;
    use winnow::{ Result};

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
        let mut input = "r#\"git branch --show-current |  sed -E \"s/(feature|develop|ver-dev|release|master|issue)(\\/.*)?/_branch_\\1/g\" \"#" ;
        let exepct = r#"git branch --show-current |  sed -E "s/(feature|develop|ver-dev|release|master|issue)(\/.*)?/_branch_\1/g" "#;
        assert_eq!(gal_raw_str(&mut input), Ok(exepct.to_string()));
        println!("{}", input);
        // 测试普通原始字符串
        let mut input = "r#\"hello\"#";
        assert_eq!(gal_raw_str(&mut input), Ok("hello".to_string()));
        println!("{}", input);

        // 测试包含特殊字符的原始字符串
        let mut input = "r#\"hell\\\"0\"#";
        let t_out = gal_raw_str(&mut input);
        println!("{}", input);
        assert_eq!(t_out, Ok(r#"hell\"0"#.to_string()));

        // 测试空字符串
        let mut input = "r#\"\"#";
        assert_eq!(gal_raw_str(&mut input), Ok("".to_string()));

        // 测试无效输入（缺少结尾标记）
        let mut input = r#"r#"hello"#;
        assert!(gal_raw_str(&mut input).is_err());
        // 测试无效输入（缺少开头标记）
        let mut input = r#""hello"\#"#;
        assert!(gal_raw_str(&mut input).is_err());

        let mut input =
            "r#\"{\"branchs\" : [{ \"name\": \"develop\" }, { \"name\" : \"release/1\"}]} \"#;";
        assert!(gal_raw_str(&mut input).is_ok());
    }

    #[test]
    fn valid_variable1() {
        let mut input = "${name}";
        assert_eq!(take_var_ref_name(&mut input), Ok("name".to_string()));
        assert_eq!(input, "");
    }
    #[test]
    fn valid_variable2() {
        let mut input = "${name[0]}";
        assert_eq!(take_var_ref_name(&mut input), Ok("name[0]".to_string()));
        assert_eq!(input, "");
    }

    #[test]
    fn trailing_characters() {
        let mut input = "${var}remaining";
        assert_eq!(take_var_ref_name(&mut input), Ok("var".to_string()));
        assert_eq!(input, "remaining");
    }

    #[test]
    fn missing_closing_brace() {
        let mut input = "${var";
        assert!(take_var_ref_name(&mut input).is_err());
    }

    #[test]
    fn no_opening_brace() {
        let mut input = "var}";
        assert!(take_var_ref_name(&mut input).is_err());
    }

    #[test]
    fn empty_variable() {
        let mut input = "${}";
        assert!(take_var_ref_name(&mut input).is_err());
    }

    #[test]
    fn special_characters_in_name() {
        let mut input = "${var-name_123}";
        assert_eq!(
            take_var_ref_name(&mut input),
            Ok("var-name_123".to_string())
        );
        assert_eq!(input, "");
    }

    #[test]
    fn nested_braces_in_name() {
        let mut input = "${var{abc}}";
        assert_eq!(take_var_ref_name(&mut input), Ok("var{abc".to_string()));
        assert_eq!(input, "}");
    }

    #[test]
    fn trimmed_spaces() {
        let mut input = "${  spaced_var  }";
        assert_eq!(take_var_ref_name(&mut input), Ok("spaced_var".to_string()));
        assert_eq!(input, "");
    }

    #[test]
    fn test_take_float() -> Result<()> {
        // 测试普通浮点数
        let mut input = "3.14";
        assert_eq!(take_float(&mut input)?, 3.14);
        assert_eq!(input, ""); // 输入被完全消耗

        // 测试整数部分为 0
        let mut input = "0.5";
        assert_eq!(take_float(&mut input)?, 0.5);

        // 测试小数部分为 0
        let mut input = "42.0";
        assert_eq!(take_float(&mut input)?, 42.0);

        // 测试缺少小数部分（无效格式）
        let mut input = "3.";
        assert!(take_float(&mut input).is_err());

        // 测试缺少小数点（无效格式）
        let mut input = "314";
        assert!(take_float(&mut input).is_err());

        // 测试非数字字符（无效格式）
        let mut input = "a.b";
        assert!(take_float(&mut input).is_err());

        Ok(())
    }
}
