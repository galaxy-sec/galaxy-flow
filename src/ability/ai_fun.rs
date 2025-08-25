use chrono::Local;
use orion_ai::{AiClient, AiClientTrait, AiConfig, AiRoleID};

use crate::ability::prelude::*;
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_error::ErrorConv;

#[derive(Clone, Debug, Default, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxAIFun {
    role: Option<String>,
    task: Option<String>,
    prompt: Option<String>,
    ai_config: Option<AiConfig>,
}

#[async_trait]
impl AsyncRunnableTrait for GxAIFun {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(ctx, vars_dict).await
    }
}

impl ComponentMeta for GxAIFun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ai_fun")
    }
}

impl GxAIFun {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    async fn execute_impl(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.ai_fun");
        let mut action = Action::from("gx.ai_fun");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());

        // 构建提示词
        let message = self.build_message(&exp)?;

        // 调用 AI 客户端
        let ai_config = self
            .ai_config()
            .clone()
            .unwrap_or(AiConfig::galaxy_load(&vars_dict.global().export().into()).err_conv()?);

        let ai_client = AiClient::new(ai_config, None).err_conv()?;
        let role = self
            .ai_role()
            .as_deref()
            .map(|r| AiRoleID::new(r.to_string()))
            .unwrap_or_else(|| ai_client.roles().default_role().clone());

        // 发送 AI 请求
        let ai_response = ai_client
            .smart_role_request(&role, message.as_str())
            .await
            .err_conv()
            .with(format!("role:{}", role))?;

        // 处理 AI 响应
        let response_content = ai_response.content;
        let response_provider = ai_response.provider.to_string();
        let timestamp = Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {}\nModel: {}\nTimestamp: {}\n",
            response_content, response_provider, timestamp
        );

        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }

    fn build_message(&self, _exp: &EnvExpress) -> Result<String, ExecReason> {
        let mut message = String::new();

        // 添加任务描述
        if let Some(task) = &self.task {
            message.push_str(&format!("任务: {}\n", task));
        }

        // 添加自定义提示词
        if let Some(prompt) = &self.prompt {
            message.push_str(&format!("提示: {}\n", prompt));
        }

        // 如果没有任务描述，使用默认的提示词
        if message.is_empty() {
            message = "请回答我的问题。".to_string();
        }

        Ok(message)
    }

    fn ai_role(&self) -> Option<&str> {
        self.role.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::{client::load_key_dict, AiConfig};
    use orion_error::TestAssert;
    use orion_variate::vars::EnvEvalable;

    use crate::{
        ability::{ability_env_init, prelude::AsyncRunnableTrait},
        evaluator::EnvExpress,
        traits::Setter,
    };

    #[tokio::test]
    async fn test_basic_ai_chat() {
        // 跳过测试，如果没有 API 密钥
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            // 如果没有 API 密钥，跳过测试
            return;
        };

        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");

        let ai_fun = crate::ability::GxAIFun::default()
            .with_ai_config(Some(config))
            .with_role(Some("developer".to_string()))
            .with_prompt(Some("请回答：1+1=?".to_string()));

        let result = ai_fun.async_exec(context, def).await;

        // 如果有 API 密钥，应该成功
        if load_key_dict("sec_deepseek_api_key").is_some() {
            result.assert();
        }
    }

    #[tokio::test]
    async fn test_task_description() {
        let ai_fun = crate::ability::GxAIFun::default()
            .with_role(Some("developer".to_string()))
            .with_task(Some("请分析以下代码问题".to_string()));

        let (context, def) = ability_env_init();

        // 测试消息构建，不实际调用 AI
        let exp = EnvExpress::from_env_mix(def.global().clone());
        let message = ai_fun.build_message(&exp).unwrap();

        assert!(message.contains("任务:"));
        assert!(message.contains("请分析以下代码问题"));
    }

    #[tokio::test]
    async fn test_empty_task_and_prompt() {
        let ai_fun = crate::ability::GxAIFun::default();
        let (context, def) = ability_env_init();

        let exp = EnvExpress::from_env_mix(def.global().clone());
        let message = ai_fun.build_message(&exp).unwrap();

        // 应该有默认提示词
        assert_eq!(message, "请回答我的问题。");
    }
}
