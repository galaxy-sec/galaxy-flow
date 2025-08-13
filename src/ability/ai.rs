use chrono::Local;
use std::path::PathBuf;

use crate::ability::prelude::*;
use crate::ai::client::AiClientTrait;
use crate::ai::provider::AiRequest;
use crate::ai::{client::AiClient, config::AiConfig};
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_error::{ErrorConv, ToStructError, UvsLogicFrom, UvsResFrom};
#[derive(Clone, Debug, Default, PartialEq, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxAIChat {
    prompt_file: Option<String>,
    prompt_msg: Option<String>,
}
#[async_trait]
impl AsyncRunnableTrait for GxAIChat {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_impl(ctx, vars_dict).await
    }
}
impl ComponentMeta for GxAIChat {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("gx.ai_chat")
    }
}

impl GxAIChat {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
    async fn execute_impl(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.shell");
        let mut action = Action::from("gx.ai_chat");
        let exp = EnvExpress::from_env_mix(vars_dict.global().clone());
        let mut message = self.prompt_msg.clone().unwrap_or("".to_string());

        if let Some(prompt_file) = &self.prompt_file {
            let prompt_file = PathBuf::from(exp.eval(prompt_file)?);
            if !prompt_file.exists() {
                return ExecReason::from_logic(format!(
                    "{path} not exists",
                    path = prompt_file.display()
                ))
                .err_result();
            }
            let data = std::fs::read_to_string(prompt_file.as_path())
                .map_err(|e| ExecReason::from_res(format!("prompt_file:{e}")))?;
            message.push('\n');
            message.push_str(data.as_str());
        }

        // call ai clien
        let ai_config = AiConfig::galaxy_load(&vars_dict.global().export().into())?;
        let ai_client = AiClient::new(ai_config).err_conv()?;

        // 创建 AI 请求
        let ai_request = AiRequest::builder()
            .model("deepseek-chat") // 使用 DeepSeek 模型
            .system_prompt("你是一个专业的AI助手，能够回答用户的问题并提供有用的建议。".to_string())
            .user_prompt(message)
            .build();

        // 发送 AI 请求
        let ai_response = ai_client.send_request(ai_request).await.err_conv()?;
        // 将 AI 响应添加到变量字典中，供后续使用
        let response_content = ai_response.content;
        let response_provider = ai_response.provider.to_string();
        let timestamp = Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {response_content}\nModel: {response_provider}\nTimestamp: {timestamp}\n"
        );

        action.finish();
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::{
        ability::{ability_env_init, ai::GxAIChat, prelude::AsyncRunnableTrait},
        traits::Setter,
        util::OptionFrom,
    };
    #[tokio::test]
    async fn ai_chat() {
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxAIChat::default().with_prompt_msg(" 1 + 1 = ?".to_opt());
        let _ = res.async_exec(context, def).await.assert();
    }
}
