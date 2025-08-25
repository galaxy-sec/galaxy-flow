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
            let tools: Vec<String> = one.1.split(",").map(String::from).collect();
            ai_fun.set_tools(tools);
        }
    }
    Ok(ai_fun)
}

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
            tools: "git-diff,git-push",
        );"#;

        match gal_ai_fun(&mut input) {
            Ok(ai_fun) => {
                assert_eq!(ai_fun.role(), &Some("developer".to_string()));
                assert_eq!(ai_fun.task(), &Some("检查 Git 状态".to_string()));
            }
            Err(e) => {
                panic!("解析失败: {:?}", e);
            }
        }
    }
}
