use super::super::prelude::*;
use super::call::action_call_args;

use crate::ability::ai_fun::GxAIFun;
use crate::parser::domain::gal_keyword;
use crate::util::OptionFrom;

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
        } else if key == "prompt" {
            ai_fun.set_prompt(one.1.to_opt());
        } else if key == "ai_config" {
            // ai_config 参数需要特殊处理，这里先简单设置
            ai_fun.set_ai_config(None); // 暂时设置为 None，后续可以根据需要扩展
        }
    }
    Ok(ai_fun)
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;

    use super::*;

    #[test]
    fn ai_fun_role_and_task() {
        let mut data = r#"
             gx.ai_fun( role: "developer", task: "检查代码质量" ) ;"#;
        let obj = gal_ai_fun(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.role(), &Some("developer".to_string()));
        assert_eq!(obj.task(), &Some("检查代码质量".to_string()));
    }

    #[test]
    fn ai_fun_with_prompt() {
        let mut data = r#"
             gx.ai_fun( role: "developer", prompt: "请分析以下代码" ) ;"#;
        let obj = gal_ai_fun(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.role(), &Some("developer".to_string()));
        assert_eq!(obj.prompt(), &Some("请分析以下代码".to_string()));
    }

    #[test]
    fn ai_fun_minimal() {
        let mut data = r#"
             gx.ai_fun( task: "简单任务" ) ;"#;
        let obj = gal_ai_fun(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.task(), &Some("简单任务".to_string()));
        assert_eq!(obj.role(), &None);
        assert_eq!(obj.prompt(), &None);
    }

    #[test]
    fn ai_fun_all_params() {
        let mut data = r#"
             gx.ai_fun( role: "developer", task: "完整任务", prompt: "完整提示" ) ;"#;
        let obj = gal_ai_fun(&mut data).assert();
        assert_eq!(data, "");
        assert_eq!(obj.role(), &Some("developer".to_string()));
        assert_eq!(obj.task(), &Some("完整任务".to_string()));
        assert_eq!(obj.prompt(), &Some("完整提示".to_string()));
    }
}
