use orion_parse::{atom::take_var_name, symbol::wn_desc};
use winnow::{
    ascii::multispace0,
    combinator::delimited,
    error::{ContextError, StrContext},
    token::{literal, take_while},
    Parser, Result,
};

pub fn take_host(input: &mut &str) -> Result<String> {
    let key =
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['.', '_', '-'])).parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_filename(input: &mut &str) -> Result<String> {
    let key =
        take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['.', '_', '-'])).parse_next(input)?;
    Ok(key.to_string())
}
pub fn take_filename_body(input: &mut &str) -> Result<String> {
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z', ['_', '-'])).parse_next(input)?;
    Ok(key.to_string())
}

// 提取受限 KEY (只包字符)
pub fn take_limit_key(input: &mut &str) -> Result<String> {
    let key = take_while(1.., ('0'..='9', 'A'..='Z', 'a'..='z')).parse_next(input)?;
    Ok(key.to_string())
}

//take a.b => (a,b)
pub fn take_dot_pair(input: &mut &str) -> Result<(String, String)> {
    // 解析第一个部分
    let first = take_var_name.parse_next(input)?;

    // 解析点号
    literal(".").parse_next(input)?;

    // 解析第二个部分
    let second = take_var_name.parse_next(input)?;

    Ok((first, second))
}

//take ${var_name}
pub fn take_var_ref_name(input: &mut &str) -> Result<String> {
    // 使用 delimited 解析 ${var_name}
    delimited(
        "${",          // 开头标记
        take_var_name, // 解析变量名
        "}",           // 结尾标记
    )
    .context(StrContext::Label("variable reference"))
    .parse_next(input)
}
//take ${var_name}
pub fn take_var_ref_fmt(input: &mut &str) -> Result<String> {
    take_var_ref_name(input).map(|x| format!("${{{x}}}"))
}

pub fn spaced<'a, P, O>(parser: P) -> impl Parser<&'a str, O, ContextError>
where
    P: Parser<&'a str, O, ContextError>,
{
    delimited(multispace0, parser, multispace0)
}
pub fn spaced_desc<'a, P, O>(parser: P, desc: &'static str) -> impl Parser<&'a str, O, ContextError>
where
    P: Parser<&'a str, O, ContextError>,
{
    delimited(multispace0, parser.context(wn_desc(desc)), multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("${var_name}", Ok("var_name"))]
    #[case("${var123}", Ok("var123"))]
    #[case("${var_name_123}", Ok("var_name_123"))]
    #[case("${var_name", Err(()))] // 缺少闭合
    #[case("var_name}", Err(()))] // 缺少开头
    #[case("${}", Err(()))] // 空变量名
    fn test_variable_reference(#[case] input: &str, #[case] expected: Result<&str, ()>) {
        let mut s = input;
        let result = take_var_ref_name(&mut s).map_err(|_| ());
        assert_eq!(result, expected.map(String::from));
    }

    #[rstest]
    #[case("a.b", Ok(("a", "b")))]
    #[case("a1.b2", Ok(("a1", "b2")))]
    #[case("a_b.c_d", Ok(("a_b", "c_d")))]
    #[case("ab", Err(()))] // 缺少点号
    #[case("a.", Err(()))] // 缺少第二部分
    #[case(".b", Err(()))] // 缺少第一部分
    fn test_dot_pairs(#[case] input: &str, #[case] expected: Result<(&str, &str), ()>) {
        let mut s = input;
        let result = take_dot_pair(&mut s);
        assert_eq!(
            result.map_err(|_| ()),
            expected.map(|(a, b)| (a.to_string(), b.to_string()))
        );
    }
}
