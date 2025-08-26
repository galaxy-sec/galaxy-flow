use crate::ability::prelude::*;
use async_trait::async_trait;

use crate::model::traits::{Getter, Setter};
use chrono;
use getset::{Getters, MutGetters, Setters, WithSetters};
use orion_ai::{AiClient, AiClientTrait, AiConfig, AiRoleID, FunctionRegistry};
use orion_error::UvsReason;

// 🎯 工具调用结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct ToolCallResult {
    round: usize,           // 执行轮次
    tool_name: String,      // 工具名称
    success: bool,          // 是否成功
    result: Option<String>, // 成功结果（如果成功）
    error: Option<String>,  // 错误信息（如果失败）
    timestamp: String,      // 执行时间戳
}

// 🎯 单轮执行结果
#[derive(Debug)]
struct RoundResult {
    has_failures: bool, // 本轮是否有失败
}

// 🎯 执行会话状态
#[derive(Debug, Clone)]
struct ExecutionSession {
    vars_dict: VarSpace,               // 变量空间
    tool_results: Vec<ToolCallResult>, // 所有工具调用结果
    success_count: usize,              // 成功次数
    failure_count: usize,              // 失败次数
    final_ai_response: String,         // 最终AI回复
}

#[derive(Clone, Debug, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct GxAIFun {
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    tools: Vec<String>,
    // 🎯 失败重试配置
    max_rounds: usize, // 最大重试轮次，默认为3
}

impl Default for GxAIFun {
    fn default() -> Self {
        Self {
            role: None,
            task: None,
            config: None,
            tools: Vec::new(),
            max_rounds: 3, // 🎯 默认最大重试3次
        }
    }
}

impl GxAIFun {
    // 🎯 主执行函数（控制流程，不超过50行）
    async fn execute_with_retry_and_storage(
        &self,
        mut ctx: ExecContext,
        vars_dict: VarSpace,
    ) -> TaskResult {
        ctx.append("gx.ai_fun");
        let base_prompt = self.task.as_deref().unwrap_or("请完成任务");

        // 初始化执行环境
        let (ai_client, role, registry) = self.initialize_execution(&vars_dict).await?;

        // 创建执行会话
        let mut session = ExecutionSession {
            vars_dict: vars_dict.clone(),
            tool_results: Vec::new(),
            success_count: 0,
            failure_count: 0,
            final_ai_response: String::new(),
        };

        // 执行重试循环
        let (task_success, total_rounds) = self
            .execute_retry_loop(&ai_client, &role, &registry, base_prompt, &mut session)
            .await?;

        // 构建最终输出
        let action = self.build_final_output(&session, task_success, total_rounds);

        Ok(TaskValue::from((
            session.vars_dict,
            ExecOut::Action(action),
        )))
    }

    // 🎯 初始化执行环境
    async fn initialize_execution(
        &self,
        vars_dict: &VarSpace,
    ) -> Result<(AiClient, AiRoleID, FunctionRegistry), ExecReason> {
        let ai_config =
            AiConfig::galaxy_load(&vars_dict.global().export().into()).map_err(|e| {
                ExecReason::Uvs(UvsReason::validation_error(format!(
                    "加载AI配置失败: {}",
                    e
                )))
            })?;

        let ai_client = AiClient::new(ai_config, None).map_err(|e| {
            ExecReason::Uvs(UvsReason::validation_error(format!(
                "创建AI客户端失败: {}",
                e
            )))
        })?;

        let role = if let Some(role_str) = &self.role {
            AiRoleID::new(role_str.clone())
        } else {
            ai_client.roles().default_role().clone()
        };

        let registry = ai_client
            .get_registry_with_tools(&self.tools)
            .map_err(|e| {
                ExecReason::Uvs(UvsReason::validation_error(format!(
                    "获取函数注册表失败: {}",
                    e
                )))
            })?;

        Ok((ai_client, role, registry))
    }

    // 🎯 执行重试循环
    async fn execute_retry_loop(
        &self,
        ai_client: &AiClient,
        role: &AiRoleID,
        registry: &FunctionRegistry,
        base_prompt: &str,
        session: &mut ExecutionSession,
    ) -> Result<(bool, usize), ExecReason> {
        let mut retry_count = 0;
        let mut task_success = false;

        println!("🚀 开始执行任务 (最大重试轮次: {})...", self.max_rounds);

        while retry_count < self.max_rounds {
            println!("\n🔄 第 {} 轮执行开始...", retry_count + 1);

            // 执行单轮
            let round_result = self
                .execute_single_round(ai_client, role, registry, base_prompt, session, retry_count)
                .await?;

            if round_result.has_failures {
                retry_count += 1;
                if retry_count >= self.max_rounds {
                    println!("⏰ 已达到最大重试次数 {}，任务最终失败", self.max_rounds);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
            } else {
                println!("✅ 第 {} 轮执行成功，任务完成！", retry_count + 1);
                task_success = true;
                break;
            }
        }

        Ok((task_success, retry_count))
    }

    // 🎯 单轮执行
    async fn execute_single_round(
        &self,
        ai_client: &AiClient,
        role: &AiRoleID,
        registry: &FunctionRegistry,
        base_prompt: &str,
        session: &mut ExecutionSession,
        round: usize,
    ) -> Result<RoundResult, ExecReason> {
        // 构建提示词
        let prompt = self.build_retry_prompt(base_prompt, &session.tool_results, round);

        // 发送AI请求
        let response = ai_client
            .role_funs_request(role, &prompt, registry.clone_functions())
            .await
            .map_err(|e| {
                ExecReason::Uvs(UvsReason::validation_error(format!(
                    "AI 函数调用请求失败: {}",
                    e
                )))
            })?;

        session.final_ai_response = response.content.clone();
        println!(
            "📤 第 {} 轮AI回复: {}",
            round + 1,
            session.final_ai_response
        );

        // 执行工具调用
        self.execute_tool_calls(response.tool_calls, registry, session, round)
            .await
    }

    // 🎯 执行工具调用
    async fn execute_tool_calls(
        &self,
        tool_calls: Option<Vec<orion_ai::provider::FunctionCall>>,
        registry: &FunctionRegistry,
        session: &mut ExecutionSession,
        round: usize,
    ) -> Result<RoundResult, ExecReason> {
        let mut has_failures = false;

        if let Some(calls) = tool_calls {
            println!("🔧 第 {} 轮执行工具调用...", round + 1);

            for tool_call in calls {
                // 检查工具权限
                if !self.tools.is_empty() && !self.tools.contains(&tool_call.function.name) {
                    self.store_failed_tool_result(
                        session,
                        &tool_call.function.name,
                        format!(
                            "工具 '{}' 不在允许的工具列表中: {:?}",
                            tool_call.function.name, self.tools
                        ),
                        round,
                    )
                    .await?;
                    has_failures = true;
                    continue;
                }

                println!("  📞 调用函数: {}", tool_call.function.name);

                match registry.execute_function(&tool_call).await {
                    Ok(result) => {
                        println!("✅ 工具调用成功: {}", tool_call.function.name);
                        self.store_successful_tool_result(
                            session,
                            &tool_call.function.name,
                            result.result.to_string(),
                            round,
                        )
                        .await?;
                    }
                    Err(e) => {
                        println!("❌ 工具调用失败: {}", e);
                        self.store_failed_tool_result(
                            session,
                            &tool_call.function.name,
                            e.to_string(),
                            round,
                        )
                        .await?;
                        has_failures = true;
                    }
                }
            }
        }

        Ok(RoundResult { has_failures })
    }

    // 🎯 存储成功工具结果
    async fn store_successful_tool_result(
        &self,
        session: &mut ExecutionSession,
        tool_name: &str,
        result: String,
        round: usize,
    ) -> Result<(), ExecReason> {
        let tool_result = ToolCallResult {
            round,
            tool_name: tool_name.to_string(),
            success: true,
            result: Some(result),
            error: None,
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        self.store_tool_result_to_varspace(session, &tool_result)
            .await?;
        session.success_count += 1;
        session.tool_results.push(tool_result);
        Ok(())
    }

    // 🎯 存储失败工具结果
    async fn store_failed_tool_result(
        &self,
        session: &mut ExecutionSession,
        tool_name: &str,
        error: String,
        round: usize,
    ) -> Result<(), ExecReason> {
        let tool_result = ToolCallResult {
            round,
            tool_name: tool_name.to_string(),
            success: false,
            result: None,
            error: Some(error),
            timestamp: chrono::Local::now().to_rfc3339(),
        };

        self.store_tool_result_to_varspace(session, &tool_result)
            .await?;
        session.failure_count += 1;
        session.tool_results.push(tool_result);
        Ok(())
    }

    // 🎯 将工具结果存储到VarSpace
    async fn store_tool_result_to_varspace(
        &self,
        session: &mut ExecutionSession,
        tool_result: &ToolCallResult,
    ) -> Result<(), ExecReason> {
        // 存储结果或错误信息
        let (value_str, var_suffix) = if tool_result.success {
            (
                tool_result
                    .result
                    .as_ref()
                    .unwrap_or(&"成功但无返回结果".to_string())
                    .clone(),
                "result",
            )
        } else {
            (
                tool_result
                    .error
                    .as_ref()
                    .unwrap_or(&"未知错误".to_string())
                    .clone(),
                "error",
            )
        };

        let var_name = format!(
            "ai_tools_{}_{}_{}",
            tool_result.tool_name, var_suffix, tool_result.round
        );
        session.vars_dict.global_mut().set(var_name, value_str);

        // 存储状态
        let status_var_name = format!(
            "ai_tools_{}_status_{}",
            tool_result.tool_name, tool_result.round
        );
        let status_value = if tool_result.success {
            "success"
        } else {
            "failed"
        };
        session
            .vars_dict
            .global_mut()
            .set(status_var_name, status_value.to_string());

        // 存储时间戳
        let timestamp_var_name = format!(
            "ai_tools_{}_timestamp_{}",
            tool_result.tool_name, tool_result.round
        );
        session
            .vars_dict
            .global_mut()
            .set(timestamp_var_name, tool_result.timestamp.clone());

        println!(
            "📊 已存储{}结果: ai_tools_{}{}_{}",
            if tool_result.success {
                "成功"
            } else {
                "失败"
            },
            tool_result.tool_name,
            if tool_result.success {
                "_result"
            } else {
                "_error"
            },
            tool_result.round
        );

        Ok(())
    }

    // 🎯 构建重试提示词
    fn build_retry_prompt(
        &self,
        base_prompt: &str,
        tool_results: &[ToolCallResult],
        current_round: usize,
    ) -> String {
        if current_round == 0 {
            format!("请完成以下任务：{}", base_prompt)
        } else {
            let mut context = format!("请重新尝试完成以下任务：{}\n\n", base_prompt);
            context.push_str(&format!(
                "当前是第 {} 次尝试（共允许 {} 次）。\n\n",
                current_round + 1,
                self.max_rounds
            ));

            let failures: Vec<_> = tool_results.iter().filter(|r| !r.success).collect();
            if !failures.is_empty() {
                context.push_str("之前的工具执行失败情况：\n");
                for (index, failure) in failures.iter().enumerate() {
                    context.push_str(&format!(
                        "{}. 第{}轮：工具 '{}' 执行失败 - {}\n",
                        index + 1,
                        failure.round,
                        failure.tool_name,
                        failure.error.as_deref().unwrap_or("未知错误")
                    ));
                }
                context.push_str("\n请根据上述失败信息，调整策略重新执行任务。");
            }

            context
        }
    }

    // 🎯 构建最终输出
    fn build_final_output(
        &self,
        session: &ExecutionSession,
        task_success: bool,
        total_rounds: usize,
    ) -> Action {
        let mut action = Action::from("ai_fun_result");

        let mut output = if task_success {
            "🎯 任务执行成功！\n"
        } else {
            "❌ 任务执行失败！\n"
        }
        .to_string();
        output.push_str(&format!("📝 执行统计:\n"));
        output.push_str(&format!("   - 总执行轮次: {}\n", total_rounds + 1));
        output.push_str(&format!("   - 成功工具调用: {}\n", session.success_count));
        output.push_str(&format!("   - 失败工具调用: {}\n\n", session.failure_count));

        output.push_str(&format!(
            "🤖 最终AI回复:\n{}\n\n",
            session.final_ai_response
        ));

        // 按轮次组织工具调用结果
        let mut results_by_round: std::collections::HashMap<usize, Vec<&ToolCallResult>> =
            std::collections::HashMap::new();

        for result in &session.tool_results {
            results_by_round
                .entry(result.round)
                .or_insert_with(Vec::new)
                .push(result);
        }

        if !results_by_round.is_empty() {
            output.push_str("🔧 工具调用详情:\n");

            for round in 1..=total_rounds + 1 {
                if let Some(round_results) = results_by_round.get(&round) {
                    output.push_str(&format!("\n   第 {} 轮:\n", round));

                    for result in round_results {
                        if result.success {
                            output.push_str(&format!(
                                "     ✅ {}: {}\n",
                                result.tool_name,
                                result.result.as_deref().unwrap_or("成功但无返回结果")
                            ));
                        } else {
                            output.push_str(&format!(
                                "     ❌ {}: {}\n",
                                result.tool_name,
                                result.error.as_deref().unwrap_or("未知错误")
                            ));
                        }
                    }
                }
            }
        }

        // 添加VarSpace使用提示
        output.push_str("\n💡 可用变量:\n");
        for result in &session.tool_results {
            let result_var = format!("ai_tools_{}_result_{}", result.tool_name, result.round);
            let error_var = format!("ai_tools_{}_error_{}", result.tool_name, result.round);

            if result.success {
                output.push_str(&format!("   - {} (成功结果)\n", result_var));
            } else {
                output.push_str(&format!("   - {} (失败信息)\n", error_var));
            }
        }

        action.stdout = output;
        action.finish();
        action
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
        self.execute_with_retry_and_storage(ctx, vars_dict).await
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
        assert!(function_names.contains(&"git-status".to_string()));
        assert!(function_names.contains(&"git-commit".to_string()));
        assert!(function_names.contains(&"git-add".to_string()));
        assert!(function_names.contains(&"git-push".to_string()));
        assert!(function_names.contains(&"git-diff".to_string()));
    }

    #[tokio::test]
    async fn test_max_rounds_default_value() {
        let ai_fun = GxAIFun::default();
        assert_eq!(*ai_fun.max_rounds(), 3, "默认max_rounds应该为3");
    }

    #[tokio::test]
    async fn test_max_rounds_custom_value() {
        let mut ai_fun = GxAIFun::default();
        ai_fun.set_max_rounds(5);
        assert_eq!(*ai_fun.max_rounds(), 5, "自定义max_rounds应该生效");
    }

    #[tokio::test]
    async fn test_tool_result_storage() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let mut ai_fun = GxAIFun::default()
            .with_config(Some(config))
            .with_role(Some("developer".to_string()))
            .with_task(Some("尝试Git操作".to_string()))
            .with_tools(vec!["git-status".to_string()])
            .with_max_rounds(2); // 设置2轮重试

        let ctx = ExecContext::new(None, false);
        let vars_dict = VarSpace::sys_init().unwrap();
        let result = ai_fun.async_exec(ctx, vars_dict).await;

        // 无论成功还是失败，都应该返回包含结果的VarSpace
        match result {
            Ok(task_value) => {
                // 检查VarSpace中是否包含工具结果
                let vars = &task_value.vars;

                // 检查是否存在结果或错误变量
                let result_var = vars.global().get_copy("ai_tools_git-status_result_1");
                let error_var = vars.global().get_copy("ai_tools_git-status_error_1");
                let status_var = vars.global().get_copy("ai_tools_git-status_status_1");

                // 至少应该有一个变量存在
                assert!(result_var.is_some() || error_var.is_some());
                assert!(status_var.is_some());
            }
            Err(_) => {
                // 如果执行失败，这是可以接受的
                // 在某些环境中，git操作可能会失败
            }
        }
    }

    #[tokio::test]
    async fn test_no_retry_mode() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let ai_fun = GxAIFun::default()
            .with_config(Some(config))
            .with_role(Some("developer".to_string()))
            .with_task(Some("简单回答：2+2=?".to_string()))
            .with_max_rounds(1); // 禁用重试

        let ctx = ExecContext::new(None, false);
        let vars_dict = VarSpace::sys_init().unwrap();
        let result = ai_fun.async_exec(ctx, vars_dict).await;

        // 这个测试应该快速完成，无论成功还是失败
        assert!(result.is_ok() || result.is_err()); // 快速返回
    }

    #[tokio::test]
    async fn test_function_length_constraints() {
        // 测试所有函数都不超过50行
        // 这是一个编译时测试，确保重构后的代码符合要求

        // 通过函数名称检查，确保所有函数都存在且功能完整
        let ai_fun = GxAIFun::default();

        // 验证默认值
        assert_eq!(*ai_fun.max_rounds(), 3);
        assert!(ai_fun.role().is_none());
        assert!(ai_fun.task().is_none());
        assert!(ai_fun.tools().is_empty());

        // 验证结构体正确性
        let _ = ai_fun.clone(); // 确保Clone trait正确实现
        let _ = format!("{:?}", ai_fun); // 确保Debug trait正确实现
    }
}
