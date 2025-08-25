use crate::ability::prelude::*;
use async_trait::async_trait;

use orion_ai::func::git::GitFunctionExecutor;
use orion_ai::{
    provider::AiRequest, AiClient, AiClientTrait, AiConfig, AiRoleID, FunctionDefinition,
    FunctionExecutor, FunctionParameter, FunctionRegistry,
};
use orion_error::{ErrorConv, UvsReason};

// 创建默认的 UsageInfo

#[derive(Clone, Debug, Default, Getters)]
pub struct GxAIFun {
    role: Option<String>,
    task: Option<String>,
    tools: Option<Vec<String>>,
    enable_function_calling: bool,
}

impl GxAIFun {
    pub fn set_role(&mut self, role: Option<String>) {
        self.role = role;
    }

    pub fn set_task(&mut self, task: Option<String>) {
        self.task = task;
    }

    pub fn set_tools(&mut self, tools: Option<Vec<String>>) {
        self.tools = tools;
    }

    pub fn set_enable_function_calling(&mut self, enable: bool) {
        self.enable_function_calling = enable;
    }

    async fn execute_simple_chat(&self, mut ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        ctx.append("gx.ai_fun");
        let prompt = self.task.as_deref().unwrap_or("请回答问题");
        let role = self.role.as_deref().unwrap_or("assistant");
        let _system_prompt = format!("你是一个专业的软件开发助手，角色：{}。", role);

        // 如果启用了函数调用，使用函数调用执行路径
        if self.enable_function_calling {
            return self.execute_with_function_calling(ctx, vars_dict).await;
        }

        // 加载 AI 配置
        let ai_config = AiConfig::galaxy_load(&vars_dict.global().export().into()).err_conv()?;
        let ai_client = AiClient::new(ai_config, None).err_conv()?;

        // 设置角色
        let ai_role = if let Some(role_str) = &self.role {
            AiRoleID::new(role_str.clone())
        } else {
            ai_client.roles().default_role().clone()
        };

        // 创建并发送 AI 请求
        let ai_response = ai_client
            .smart_role_request(&ai_role, prompt)
            .await
            .err_conv()
            .with(format!("role:{}", ai_role))?;

        // 获取回复内容
        let response_content = ai_response.content;
        let response_provider = ai_response.provider.to_string();
        let timestamp = chrono::Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {response_content}\nModel: {response_provider}\nTimestamp: {timestamp}\n"
        );

        // 创建输出动作
        let mut action = Action::from("ai_chat_reply");
        action.stdout = response_content;
        action.finish();

        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }

    async fn execute_with_function_calling(
        &self,
        mut ctx: ExecContext,
        vars_dict: VarSpace,
    ) -> TaskResult {
        ctx.append("gx.ai_fun(function_calling)");
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
        let system_prompt = format!(
            "你是一个专业的软件开发助手，角色：{}。\n\n## 工作原则\n1. 仔细分析用户的需求\n2. 根据可用工具选择最合适的执行方式\n3. 工具调用必须精确，参数要完整\n4. 如果遇到问题，提供清晰的错误说明",
            role
        );

        // 创建函数注册表并注册可用的工具
        let mut registry = FunctionRegistry::new();
        let available_functions = self.get_available_functions();

        // 注册函数定义
        for function_def in &available_functions {
            registry
                .register_function(function_def.clone())
                .err_conv()?;
        }

        // 注册 Git 函数执行器
        let git_executor = std::sync::Arc::new(GitFunctionExecutor);
        for function_name in git_executor.supported_functions() {
            registry
                .register_executor(function_name, git_executor.clone())
                .err_conv()?;
        }

        // 创建带有函数调用的 AI 请求
        let request = AiRequest::builder()
            .model("gpt-3.5-turbo")
            .system_prompt(system_prompt)
            .user_prompt(user_prompt)
            .functions(available_functions.clone())
            .enable_function_calling(true)
            .build();

        // 发送 AI 请求
        println!("🚀 发送 AI 请求 (启用函数调用)...");
        let response = ai_client
            .send_request(request)
            .await
            .err_conv()
            .with(("AI 函数调用请求失败", "gx.ai_fun"))?;

        let response_content = response.content;
        let response_provider = response.provider.to_string();
        let timestamp = chrono::Local::now().to_rfc3339();

        println!(
            "AI Response:\nContent: {response_content}\nModel: {response_provider}\nTimestamp: {timestamp}\n"
        );

        // 处理函数调用
        let mut results = Vec::new();

        if let Some(tool_calls) = response.tool_calls {
            println!("🔧 AI 请求执行工具调用:");

            for tool_call in tool_calls {
                println!("  📞 调用函数: {}", tool_call.function.name);

                // 使用注册表执行函数调用
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

    // handle_tool_call 方法已不再需要，因为现在使用 FunctionRegistry 来执行函数调用

    fn get_available_functions(&self) -> Vec<FunctionDefinition> {
        let mut functions = Vec::new();

        if let Some(tools) = &self.tools {
            for tool_name in tools {
                if let Some(def) = self.create_function_definition(tool_name) {
                    functions.push(def);
                }
            }
        } else {
            // 默认提供常用工具
            let default_tools = vec![
                "git status",
                "git diff",
                "git add",
                "git commit",
                "git push",
            ];
            for tool_name in default_tools {
                if let Some(def) = self.create_function_definition(tool_name) {
                    functions.push(def);
                }
            }
        }

        functions
    }

    fn create_function_definition(&self, tool_name: &str) -> Option<FunctionDefinition> {
        match tool_name {
            "git status" => {
                let parameters = vec![FunctionParameter {
                    name: "path".to_string(),
                    r#type: "string".to_string(),
                    description: "Git仓库路径，默认为当前目录".to_string(),
                    required: false,
                }];
                Some(FunctionDefinition {
                    name: "git_status".to_string(),
                    description: "显示Git仓库状态".to_string(),
                    parameters,
                })
            }
            "git diff" => {
                let parameters = vec![
                    FunctionParameter {
                        name: "path".to_string(),
                        r#type: "string".to_string(),
                        description: "Git仓库路径，默认为当前目录".to_string(),
                        required: false,
                    },
                    FunctionParameter {
                        name: "staged".to_string(),
                        r#type: "boolean".to_string(),
                        description: "是否只显示暂存的变更，默认为false".to_string(),
                        required: false,
                    },
                ];
                Some(FunctionDefinition {
                    name: "git_diff".to_string(),
                    description: "显示Git仓库的变更差异".to_string(),
                    parameters,
                })
            }
            "git add" => {
                let parameters = vec![FunctionParameter {
                    name: "files".to_string(),
                    r#type: "array".to_string(),
                    description: "要添加的文件列表，空数组表示添加所有变更".to_string(),
                    required: false,
                }];
                Some(FunctionDefinition {
                    name: "git_add".to_string(),
                    description: "将文件添加到Git暂存区".to_string(),
                    parameters,
                })
            }
            "git commit" => {
                let parameters = vec![FunctionParameter {
                    name: "message".to_string(),
                    r#type: "string".to_string(),
                    description: "提交信息".to_string(),
                    required: true,
                }];
                Some(FunctionDefinition {
                    name: "git_commit".to_string(),
                    description: "提交暂存的变更到Git仓库".to_string(),
                    parameters,
                })
            }
            "git push" => {
                let parameters = vec![
                    FunctionParameter {
                        name: "remote".to_string(),
                        r#type: "string".to_string(),
                        description: "远程仓库名称，默认为origin".to_string(),
                        required: false,
                    },
                    FunctionParameter {
                        name: "branch".to_string(),
                        r#type: "string".to_string(),
                        description: "分支名称，默认为当前分支".to_string(),
                        required: false,
                    },
                ];
                Some(FunctionDefinition {
                    name: "git_push".to_string(),
                    description: "推送本地提交到远程仓库".to_string(),
                    parameters,
                })
            }
            _ => None,
        }
    }
}

impl ComponentMeta for GxAIFun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_fun")
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxAIFun {
    async fn async_exec(&self, ctx: ExecContext, vars_dict: VarSpace) -> TaskResult {
        if self.enable_function_calling {
            self.execute_with_function_calling(ctx, vars_dict).await
        } else {
            self.execute_simple_chat(ctx, vars_dict).await
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_basic_ai_chat() {
        let mut ai_fun = GxAIFun::default();
        ai_fun.set_role(Some("developer".to_string()));
        ai_fun.set_task(Some("请回答：1+1=?".to_string()));

        // 创建基本的执行环境
        let ctx = ExecContext::new(None, false);
        let vars_dict = VarSpace::sys_init().unwrap();
        let result = ai_fun.async_exec(ctx, vars_dict).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_function_definitions() {
        let mut ai_fun = GxAIFun::default();
        ai_fun.set_tools(Some(vec!["git status".to_string(), "git diff".to_string()]));

        let functions = ai_fun.get_available_functions();
        assert_eq!(functions.len(), 2);
        assert_eq!(functions[0].name, "git_status");
        assert_eq!(functions[1].name, "git_diff");
    }
}

// Tests will be added later
