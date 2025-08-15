use chrono::Local;
use std::path::PathBuf;

use crate::ability::prelude::*;
use crate::ai::client::AiClientTrait;
use crate::ai::{client::AiClient, config::AiConfig};
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_error::{ErrorConv, ToStructError, UvsResFrom};
#[derive(Clone, Debug, Default,  Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxAIChat {
    prompt_file: Option<String>,
    prompt_msg: Option<String>,
    ai_config: Option<AiConfig>,
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
                return ExecReason::Gxl(format!("{path} not exists", path = prompt_file.display()))
                    .err_result();
            }
            let data = std::fs::read_to_string(prompt_file.as_path())
                .map_err(|e| ExecReason::from_res(format!("prompt_file:{e}")))?;
            message.push('\n');
            message.push_str(data.as_str());
        }

        // call ai clien
        let ai_config = self.ai_config().clone().unwrap_or(AiConfig::galaxy_load(&vars_dict.global().export().into())?);
        let ai_client = AiClient::new(ai_config).err_conv()?;
        let role = ai_client.roles().default_role;
        //ai_config
        let ai_response = ai_client
            .smart_role_request(role, message.as_str())
            .await
            .err_conv()
            .with(format!("role:{role}"))?;

        // 发送 AI 请求
        //let ai_response = ai_client.send_request(ai_request).await.err_conv()?;
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
    use orion_variate::vars::EnvEvalable;

    use crate::{
        ability::{ability_env_init, ai::GxAIChat, prelude::AsyncRunnableTrait}, ai::{client::load_key_dict, AiConfig}, traits::Setter, util::OptionFrom
    };
    #[tokio::test]
    async fn ai_chat() {
     let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        AiConfig::example().env_eval(&dict)
    } else {
        return;
    };
        let (context, mut def) = ability_env_init();
        def.global_mut()
            .set("CONF_ROOT", "${GXL_PRJ_ROOT}/tests/material");
        let res = GxAIChat::default().with_ai_config(Some(config)).with_prompt_msg(" 1 + 1 = ?".to_opt());
        let _ = res.async_exec(context, def).await.assert();
    }
}
