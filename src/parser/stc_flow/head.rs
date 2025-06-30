use crate::parser::atom::{spaced_desc, take_var_ref_fmt};
use crate::parser::prelude::*;
use orion_parse::atom::take_var_name;
use winnow::{
    ascii::multispace0,
    combinator::{preceded, separated},
    Parser,
};

#[derive(Default, Debug, Clone, Getters)]
pub struct FlowHeadDto {
    pub keyword: String,
    pub first: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
    pub args: Vec<(String, String)>,
}

/// 解析流头部声明
pub fn galaxy_flow_head(input: &mut &str) -> ModalResult<FlowHeadDto> {
    spaced_desc("flow", "<keyword:flow>").parse_next(input)?;

    // 解析初始管道分隔列表
    let mut initial_list = parse_initial_list(input)?;
    multispace0(input)?;

    // 解析流程名称并确定语法类型
    let (flow_name, syntax_type) = parse_flow_name(input, &mut initial_list)?;

    // 根据语法类型解析后续部分
    match syntax_type {
        SyntaxType::New => parse_new_syntax(input, flow_name, initial_list),
        SyntaxType::Old => parse_old_syntax(input, flow_name),
        SyntaxType::Implicit => Ok(build_dto(flow_name, Vec::new(), initial_list)),
    }
}

// 语法类型枚举
enum SyntaxType {
    New,      // 新语法（管道分隔）
    Old,      // 旧语法（冒号分隔）
    Implicit, // 隐式语法（无分隔符）
}

/// 解析初始的管道分隔列表
fn parse_initial_list(input: &mut &str) -> ModalResult<Vec<String>> {
    separated(
        0..,
        alt((take_var_path, take_var_ref_fmt)),
        (multispace0, '|', multispace0),
    )
    .context(wn_desc("<pre-flow>"))
    .parse_next(input)
}

/// 解析流程名称并确定语法类型
fn parse_flow_name(
    input: &mut &str,
    initial_list: &mut Vec<String>,
) -> ModalResult<(String, SyntaxType)> {
    Ok(if starts_with("@", input) {
        let (_, name) = spaced_desc(("@", take_var_name), "<flow-name>").parse_next(input)?;
        (name, SyntaxType::New)
    } else if starts_with("|", input) {
        spaced_desc("|", "|").parse_next(input)?;
        let (_, name) = spaced_desc(("@", take_var_name), "<flow-name>").parse_next(input)?;
        (name, SyntaxType::New)
    } else if starts_with(":", input) {
        let name = pop_first_or_fail(initial_list, "<flow-name>")?;
        (name, SyntaxType::Old)
    } else {
        let name = pop_first_or_fail(initial_list, "<flow-name>")?;
        (name, SyntaxType::Implicit)
    })
}

/// 从列表中安全取出第一个元素或返回错误
fn pop_first_or_fail(list: &mut Vec<String>, context: &'static str) -> ModalResult<String> {
    if !list.is_empty() {
        return Ok(list.remove(0));
    }
    fail.context(wn_desc(context)).parse_next(&mut "")?;
    Ok(String::new())
}

/// 解析新语法（管道分隔）的后续部分
fn parse_new_syntax(
    input: &mut &str,
    flow_name: String,
    before: Vec<String>,
) -> ModalResult<FlowHeadDto> {
    multispace0(input)?;
    let after = parse_pipe_separated_list(input, "<next-flow>")?;
    Ok(build_dto(flow_name, before, after))
}

/// 解析旧语法（冒号分隔）的后续部分
fn parse_old_syntax(input: &mut &str, flow_name: String) -> ModalResult<FlowHeadDto> {
    let before = parse_colon_separated_list(input, "<pre-flow>")?;
    multispace0(input)?;
    let after = parse_colon_separated_list(input, "<next-flow>")?;
    Ok(build_dto(flow_name, before, after))
}

/// 解析管道分隔的列表
fn parse_pipe_separated_list(input: &mut &str, context: &'static str) -> ModalResult<Vec<String>> {
    if starts_with("|", input) {
        preceded(
            ('|', multispace0),
            separated(
                0..,
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, '|', multispace0),
            ),
        )
        .context(wn_desc(context))
        .parse_next(input)
    } else {
        Ok(Vec::new())
    }
}

/// 解析冒号分隔的列表
fn parse_colon_separated_list(input: &mut &str, context: &'static str) -> ModalResult<Vec<String>> {
    if starts_with(":", input) {
        preceded(
            (multispace0, ':', multispace0),
            separated(
                0..,
                alt((take_var_path, take_var_ref_fmt)),
                (multispace0, ',', multispace0),
            ),
        )
        .context(wn_desc(context))
        .parse_next(input)
    } else {
        Ok(Vec::new())
    }
}

/// 构建DTO对象
fn build_dto(flow_name: String, before: Vec<String>, after: Vec<String>) -> FlowHeadDto {
    FlowHeadDto {
        keyword: "flow".to_string(),
        first: flow_name,
        before,
        after,
        args: Vec::new(),
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_take_flow_head_v1() {
        // Set up a parse context and relevant variables to pass in to the function
        let main_key: &str = "flow";
        let mut input: &str = r#"flow chatbot : b_flow1,b_flow2 : a_flow1,a_flow2"#;

        // Pass variables into the function and capture the results
        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };

        // Expected results
    }

    #[test]
    fn test_take_flow_head_v2_0() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow b_flow1 | b_flow2 | @chatbot | a_flow1 | a_flow2"#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };

        // Expected results
    }

    #[test]
    fn test_take_flow_head_v2_1() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow chatbot |  a_flow1| a_flow2"#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert!(dto.before.is_empty());
                assert_eq!(dto.after, ["a_flow1", "a_flow2"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };
    }

    #[test]
    fn test_take_flow_head_v2_2() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow b_flow1| b_flow2 | @chatbot "#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "chatbot");
                assert_eq!(dto.before, ["b_flow1", "b_flow2"]);
                assert!(dto.after.is_empty());
            }
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };
    }
    #[test]
    fn test_take_flow_head_v2_3() {
        let main_key: &str = "flow";
        let mut input: &str = r#"flow lint | rust_flow.lint "#;

        match galaxy_flow_head(&mut input) {
            Ok(dto) => {
                assert_eq!(dto.keyword, main_key);
                assert_eq!(dto.first, "lint");
                assert_eq!(dto.after, ["rust_flow.lint"]);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                panic!();
            }
        };
    }
}
