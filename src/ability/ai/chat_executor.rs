use chrono::Local;
use orion_ai::{client::AiClientBuilder, provider::AiResponse, AiClientTrait, AiConfig, AiRoleID};
use orion_error::{ErrorConv, UvsConfFrom};
use std::path::PathBuf;

use crate::ability::prelude::*;
use crate::model::traits::Setter;
use getset::{Getters, MutGetters, Setters};
use orion_error::{ToStructError, UvsResFrom};
use orion_sec::sec::{SecFrom, SecValueType};

/// AI聊天执行器，专注于AI对话功能
///
/// 这个结构体提供了简化的AI对话接口，支持从文件或直接消息中
/// 获取提示词，与AI进行交互。
///
/// # 示例
///
/// ```rust
/// let executor = ChatExecutor {
///     role: Some("developer".to_string()),
///     prompt_msg: Some("1 + 1 = ?".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, Default, Getters, MutGetters, Setters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct ChatExecutor {
    prompt_file: Option<String>,
    prompt_msg: Option<String>,
    config: Option<AiConfig>,
    role: Option<String>,
}

impl ChatExecutor {
    /// 执行AI聊天任务
    ///
    /// 这是主要的执行入口点，负责协调AI对话的整个执行流程。
    pub async fn execute(&self, mut ctx: ExecContext, mut vars: VarSpace) -> TaskResult {
        // 设置执行上下文
        ctx.append("gx.ai_chat");
        let mut action = Action::from("gx.ai_chat");
        let exp = EnvExpress::from_env_mix(vars.global().clone());
        let mut message = self.prompt_msg.clone().unwrap_or("".to_string());

        // 如果有提示文件，读取文件内容
        if let Some(prompt_file) = &self.prompt_file {
            let prompt_file = PathBuf::from(exp.eval(prompt_file)?);
            if !prompt_file.exists() {
                return ExecReason::Gxl(format!("{path} not exists", path = prompt_file.display()))
                    .err_result();
            }
            let data = std::fs::read_to_string(prompt_file.as_path())
                .map_err(|e| ExecReason::from_res(format!("prompt_file:{e}")))?;
            message.push('\n');
            message.push_str(data.as_str());
        }

        // 执行AI对话
        let response = self.execute_chat(&message, &vars).await?;

        // 存储结果
        vars.global_mut().set(
            "AI".to_string(),
            SecValueType::nor_from(response.content.clone()),
        );

        action.finish();
        Ok(TaskValue::from((vars, ExecOut::Action(action))))
    }

    /// 执行AI对话
    ///
    /// 初始化AI客户端并发送对话请求。
    async fn execute_chat(&self, message: &str, vars: &VarSpace) -> ExecResult<AiResponse> {
        // 加载AI配置
        let ai_config = self.config().clone().unwrap_or(
            AiConfig::galaxy_load(&vars.global().export().into())
                .err_conv()
                .want("load ai config")?,
        );

        // 创建AI客户端
        let ai_client = AiClientBuilder::new(ai_config)
            .with_timout(60)
            .build()
            .err_conv()?;

        // 设置角色
        let role = self
            .role()
            .as_ref()
            .map(AiRoleID::new)
            .unwrap_or(ai_client.roles().default_role().clone());

        // 发送AI请求
        let ai_response = ai_client
            .smart_role_request(&role, message)
            .await
            .map_err(|e| ExecReason::from_conf(format!("AI请求失败: {}", e)))?;

        // 记录响应信息
        let response_content = &ai_response.content;
        let response_provider = ai_response.provider.to_string();
        let timestamp = Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {response_content}\nModel: {response_provider}\nTimestamp: {timestamp}\n"
        );

        Ok(ai_response)
    }
}

impl ComponentMeta for ChatExecutor {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ai_chat")
    }
}

#[async_trait]
impl AsyncRunnableTrait for ChatExecutor {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        self.execute(ctx, vars).await
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::{client::load_key_dict, AiConfig};
    use orion_error::TestAssert;
    use orion_variate::vars::EnvEvalable;

    use crate::{
        ability::{ability_env_init, prelude::AsyncRunnableTrait},
        traits::Setter,
    };

    use super::*;

    #[tokio::test]
    async fn test_basic_chat() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let mut executor = ChatExecutor::default();
        executor.set_config(Some(config));
        executor.set_prompt_msg(Some("1 + 1 = ?".to_string()));

        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");

        let result = executor.async_exec(context, def).await;
        result.assert();
    }

    #[tokio::test]
    async fn test_chat_with_role() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let mut executor = ChatExecutor::default();
        executor.set_config(Some(config));
        executor.set_role(Some("developer".to_string()));
        executor.set_prompt_msg(Some("解释什么是人工智能".to_string()));

        let (context, def) = ability_env_init();
        let result = executor.async_exec(context, def).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_default_values() {
        let executor = ChatExecutor::default();

        assert!(executor.prompt_file().is_none());
        assert!(executor.prompt_msg().is_none());
        assert!(executor.config().is_none());
        assert!(executor.role().is_none());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let mut executor = ChatExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_prompt_msg(Some("test message".to_string()));
        executor.set_prompt_file(Some("test.txt".to_string()));

        assert_eq!(executor.role(), &Some("developer".to_string()));
        assert_eq!(executor.prompt_msg(), &Some("test message".to_string()));
        assert_eq!(executor.prompt_file(), &Some("test.txt".to_string()));
    }

    #[tokio::test]
    async fn test_component_meta() {
        let executor = ChatExecutor::default();
        let meta = executor.gxl_meta();

        assert_eq!(meta, GxlMeta::from("gx.ai_chat"));
    }

    #[tokio::test]
    async fn test_clone_and_debug() {
        let mut executor = ChatExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_prompt_msg(Some("test message".to_string()));

        // 测试Clone trait
        let cloned = executor.clone();
        assert_eq!(cloned.role(), executor.role());
        assert_eq!(cloned.prompt_msg(), executor.prompt_msg());

        // 测试Debug trait
        let debug_str = format!("{:?}", executor);
        assert!(debug_str.contains("ChatExecutor"));
    }
}
