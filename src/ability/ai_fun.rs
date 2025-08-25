use crate::ability::prelude::*;
use async_trait::async_trait;

use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_ai::{AiClient, AiClientTrait, AiConfig, AiRoleID};
use orion_error::{ErrorConv, UvsReason};

// 创建默认的 UsageInfo

#[derive(Clone, Debug, Default, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxAIFun {
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    enable_function_calling: bool,
    tools: String, // 字符串格式：如 "git-diff,git-push"
}

impl GxAIFun {
    async fn execute_with_function_calling(
        &self,
        mut ctx: ExecContext,
        vars_dict: VarSpace,
    ) -> TaskResult {
        ctx.append("gx.ai_fun");
        let prompt = self.task.as_deref().unwrap_or("请完成任务");

        let user_prompt = format!("请完成以下任务：{}", prompt);

        // 加载 AI 配置
        let ai_config = AiConfig::galaxy_load(&vars_dict.global().export().into()).err_conv()?;
        let ai_client = AiClient::new(ai_config, None).err_conv()?;

        // 设置角色
        let role = if let Some(role_str) = &self.role {
            AiRoleID::new(role_str.clone())
        } else {
            ai_client.roles().default_role().clone()
        };

        // 🎯 获取根据工具列表过滤的注册表
        let registry = ai_client.get_filtered_registry(&self.tools).err_conv()?;
        let available_functions = registry.clone_functions();

        // 发送 AI 请求
        println!("🚀 发送 AI 请求 (启用工具过滤)...");
        let response = ai_client
            .role_funs_request(&role, user_prompt.as_str(), available_functions)
            .await
            .err_conv()
            .with(("AI 函数调用请求失败", "gx.ai_fun"))?;

        let response_content = response.content;
        let response_provider = response.provider.to_string();
        let timestamp = chrono::Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {response_content}\nModel: {response_provider}\nTimestamp: {timestamp}\n"
        );

        // 🎯 使用过滤后的注册表执行函数调用
        let mut results = Vec::new();

        if let Some(tool_calls) = response.tool_calls {
            if self.tools.is_empty() {
                println!("🔧 AI 请求执行工具调用 (所有可用工具):");
            } else {
                println!("🔧 AI 请求执行工具调用 (指定工具: {:?}):", self.tools);
            }

            for tool_call in tool_calls {
                // 🎯 检查函数是否在允许的 tools 列表中
                if !self.tools.is_empty() && !self.tools.contains(&tool_call.function.name) {
                    let error_msg = format!(
                        "工具 '{}' 不在允许的工具列表中: {:?}",
                        tool_call.function.name, self.tools
                    );
                    println!("❌ {}", error_msg);
                    return Err(ExecReason::Uvs(UvsReason::validation_error(error_msg)).into());
                }

                println!("  📞 调用函数: {}", tool_call.function.name);

                match registry.execute_function(&tool_call).await {
                    Ok(result) => {
                        println!("✅ 工具调用成功: {}", tool_call.function.name);
                        println!("   📤 结果: {}", result.result);
                        results.push(format!(
                            "工具调用完成: {} - {}",
                            tool_call.function.name, result.result
                        ));
                    }
                    Err(e) => {
                        println!("❌ 工具调用失败: {}", e);
                        return Err(ExecReason::Uvs(UvsReason::validation_error(format!(
                            "工具调用执行失败: {} - {}",
                            tool_call.function.name, e
                        )))
                        .into());
                    }
                }
            }
        }

        // 创建输出动作
        let mut action = Action::from("ai_fun_result");
        if results.is_empty() {
            action.stdout = response_content;
        } else {
            let all_results = results.join("\n");
            action.stdout = format!(
                "AI 回复: {}\n\n执行结果:\n{}",
                response_content, all_results
            );
        }
        action.finish();

        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }

    // 🎉 这些方法已不再需要，因为现在使用全局注册表，所有函数在启动时已预注册
    // get_available_functions 和 create_function_definition 方法已被移除
    // 函数定义现在由 GlobalFunctionRegistry 统一管理
}

impl ComponentMeta for GxAIFun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_fun")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxAIFun {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        self.execute_with_function_calling(ctx, vars_dict).await
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::client::load_key_dict;
    use orion_ai::GlobalFunctionRegistry;
    use orion_variate::vars::EnvEvalable;

    use super::*;

    #[tokio::test]
    async fn test_basic_ai_chat() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };
        let mut ai_fun = GxAIFun::default().with_config(Some(config));
        ai_fun.set_role(Some("developer".to_string()));
        ai_fun.set_task(Some("请回答：1+1=?".to_string()));

        // 创建基本的执行环境
        let ctx = ExecContext::new(None, false);
        let vars_dict = VarSpace::sys_init().unwrap();
        let result = ai_fun.async_exec(ctx, vars_dict).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_global_registry_initialization() {
        // 重置注册表（用于测试）
        GlobalFunctionRegistry::reset();

        // 初始化注册表
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 获取注册表副本
        let registry = GlobalFunctionRegistry::get_registry();
        assert!(registry.is_ok());

        let registry = registry.unwrap();
        let function_names = registry.get_supported_function_names();

        // 验证Git工具已注册
        assert!(function_names.contains(&"git_status".to_string()));
        assert!(function_names.contains(&"git_commit".to_string()));
        assert!(function_names.contains(&"git_add".to_string()));
        assert!(function_names.contains(&"git_push".to_string()));
        assert!(function_names.contains(&"git_diff".to_string()));
    }
}

// Tests will be added later
