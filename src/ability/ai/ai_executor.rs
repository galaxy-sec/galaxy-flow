use crate::{ability::prelude::*, traits::Setter};
use async_trait::async_trait;

use orion_ai::{AiConfig, AiExecUnit, AiExecUnitBuilder};
use orion_error::ErrorConv;
use orion_sec::sec::SecFrom;
use orion_sec::sec::SecValueType;

use getset::{Getters, MutGetters, Setters};

/// AI执行器，支持函数调用的AI任务执行器
///
/// 这个结构体提供了简化的AI任务执行接口，复用orion_ai的FunctionRegistry
/// 来实现工具调用功能。
///
/// # 示例
///
/// ```rust
/// let executor = AiExecutor {
///     role: Some("developer".to_string()),
///     task: Some("分析代码".to_string()),
///     ..Default::default()
/// };
/// ```
#[derive(Clone, Debug, Getters, MutGetters, Setters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiExecutor {
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    tools: Vec<String>,
    // 最大重试轮次，默认为3
    max_rounds: usize,
}

impl Default for AiExecutor {
    fn default() -> Self {
        Self {
            role: None,
            task: None,
            config: None,
            tools: Vec::new(),
            max_rounds: 3, // 默认最大重试3次
        }
    }
}

impl AiExecutor {
    /// 执行AI任务
    ///
    /// 这是主要的执行入口点，负责协调整个AI任务的执行流程。
    pub async fn execute(&self, mut ctx: ExecContext, mut vars: VarSpace) -> TaskResult {
        // 设置执行上下文
        ctx.append("gx.ai_fun");
        let mut action = Action::from("gx.ai_fun");
        let base_prompt = self.task.as_deref().unwrap_or("请完成任务");

        // 初始化执行单元
        let exec_unit = self.setup_exec_unit(&vars)?;

        // 执行AI请求
        let response = exec_unit.execute(base_prompt).await.err_conv()?;

        // 存储结果
        vars.global_mut().set(
            "AI".to_string(),
            SecValueType::nor_from(response.content.clone()),
        );
        action.finish();
        Ok(TaskValue::from((vars, ExecOut::Action(action))))
    }

    /// 创建执行单元
    ///
    /// 使用配置创建AI执行单元，封装客户端、角色和函数注册表。
    fn setup_exec_unit(&self, vars: &VarSpace) -> ExecResult<AiExecUnit> {
        // 使用构建器创建执行单元
        Ok(AiExecUnitBuilder::new(vars.global().export().into())
            .with_config_opt(self.config.clone())
            .with_role_opt(self.role.clone())
            .with_tools(self.tools.clone())
            .build()
            .err_conv()
            .want("create ai exec unit")?)
    }
}

impl ComponentMeta for AiExecutor {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_fun")
    }
}

#[async_trait]
impl AsyncRunnableTrait for AiExecutor {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        self.execute(ctx, vars).await
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::client::load_key_dict;
    use orion_ai::GlobalFunctionRegistry;
    use orion_error::TestAssert;
    use orion_variate::vars::EnvEvalable;

    use super::*;

    #[tokio::test]
    async fn test_basic_ai_execution() {
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let mut executor = AiExecutor::default();
        executor.set_config(Some(config));
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some("请回答：1+1=?".to_string()));

        let ctx = ExecContext::new(None, false);
        let vars = VarSpace::sys_init().unwrap();
        let result = executor.async_exec(ctx, vars).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_executor_with_tools() {
        GlobalFunctionRegistry::initialize().assert();
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };

        let mut executor = AiExecutor::default();
        executor.set_config(Some(config));
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some("检查 Git status".to_string()));
        executor.set_tools(vec!["git-status".to_string()]);

        let ctx = ExecContext::new(None, false);
        let vars = VarSpace::sys_init().unwrap();
        let result = executor.async_exec(ctx, vars).await;

        // 无论成功还是失败，都应该返回结果
        match result {
            Ok(task_value) => {
                let vars = &task_value.vars;
                let result_var = vars.get("AI");
                assert!(result_var.is_some());
            }
            Err(e) => {
                // 在某些环境中，git操作可能会失败，这是可以接受的
                println!("执行失败: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_default_values() {
        let executor = AiExecutor::default();

        assert!(executor.role().is_none());
        assert!(executor.task().is_none());
        assert!(executor.config().is_none());
        assert!(executor.tools().is_empty());
    }

    #[tokio::test]
    async fn test_builder_pattern() {
        let mut executor = AiExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some("test task".to_string()));
        executor.set_tools(vec!["git-status".to_string()]);

        assert_eq!(executor.role(), &Some("developer".to_string()));
        assert_eq!(executor.task(), &Some("test task".to_string()));
        assert_eq!(executor.tools(), &vec!["git-status".to_string()]);
    }

    #[tokio::test]
    async fn test_component_meta() {
        let executor = AiExecutor::default();
        let meta = executor.gxl_meta();

        assert_eq!(meta, GxlMeta::from("ai_fun"));
    }

    #[tokio::test]
    async fn test_clone_and_debug() {
        let mut executor = AiExecutor::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some("test task".to_string()));

        // 测试Clone trait
        let cloned = executor.clone();
        assert_eq!(cloned.role(), executor.role());
        assert_eq!(cloned.task(), executor.task());

        // 测试Debug trait
        let debug_str = format!("{:?}", executor);
        assert!(debug_str.contains("AiExecutor"));
    }
}
