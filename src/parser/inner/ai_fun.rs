use orion_variate::opt::OptionFrom;

use crate::ability::ai_fun::GxAIFun;
use crate::parser::inner::prelude::*;

pub fn gal_ai_fun(input: &mut &str) -> Result<GxAIFun> {
    let mut ai_fun = GxAIFun::default();
    gal_keyword("gx.ai_fun", input)?;
    let props = action_call_args.parse_next(input)?;
    for one in props {
        let key = one.0.to_lowercase();
        if key == "role" {
            ai_fun.set_role(one.1.to_opt());
        } else if key == "task" {
            ai_fun.set_task(one.1.to_opt());
        } else if key == "tools" {
            if let Some(tools_str) = one.1.to_opt() {
                let tools = parse_tools_list(&tools_str);
                ai_fun.set_tools(Some(tools));
            }
        } else if key == "enable_function_calling" {
            if let Some(enable_str) = one.1.to_opt() {
                let enable = parse_bool(&enable_str);
                ai_fun.set_enable_function_calling(enable);
            }
        }
    }
    Ok(ai_fun)
}

fn parse_tools_list(tools_str: &str) -> Vec<String> {
    // 解析类似 ["git status", "git diff"] 的列表
    let tools_str = tools_str.trim();
    if tools_str.starts_with('[') && tools_str.ends_with(']') {
        let content = &tools_str[1..tools_str.len() - 1];
        content
            .split(',')
            .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
            .filter(|s| !s.is_empty())
            .collect()
    } else {
        // 如果不是列表格式，当作单个工具处理
        vec![tools_str.to_string()]
    }
}

fn parse_bool(bool_str: &str) -> bool {
    bool_str.trim().to_lowercase() == "true"
}

#[cfg(test)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ai_fun_with_role() {
        let mut input = r#"gx.ai_fun(
            role: "developer",
            task: "检查代码"
        );"#;

        let ai_fun = gal_ai_fun(&mut input).unwrap();

        assert_eq!(ai_fun.role(), &Some("developer".to_string()));
        assert_eq!(ai_fun.task(), &Some("检查代码".to_string()));
    }

    #[test]
    fn test_parse_ai_fun_with_tools() {
        let mut input = r#"gx.ai_fun(
            role: "developer",
            task: "检查 Git 状态",
            enable_function_calling: "true"
        );"#;

        match gal_ai_fun(&mut input) {
            Ok(ai_fun) => {
                assert_eq!(ai_fun.role(), &Some("developer".to_string()));
                assert_eq!(ai_fun.task(), &Some("检查 Git 状态".to_string()));
                assert_eq!(ai_fun.enable_function_calling(), &true);
            }
            Err(e) => {
                panic!("解析失败: {:?}", e);
            }
        }
    }

    #[test]
    fn test_parse_tools_list() {
        let tools = parse_tools_list(r#"["git status", "git diff"]"#);
        assert_eq!(tools, vec!["git status", "git diff"]);

        let single_tool = parse_tools_list("git status");
        assert_eq!(single_tool, vec!["git status"]);
    }

    #[test]
    fn test_parse_bool() {
        assert_eq!(parse_bool("true"), true);
        assert_eq!(parse_bool("false"), false);
        assert_eq!(parse_bool("TRUE"), true);
        assert_eq!(parse_bool("FALSE"), false);
    }
}
