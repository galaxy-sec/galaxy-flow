use crate::{ability::prelude::*, traits::Setter};
use async_trait::async_trait;

use orion_ai::{AiConfig, AiExecUnit, AiExecUnitBuilder};
use orion_error::ErrorConv;
use orion_sec::sec::SecFrom;
use orion_sec::sec::SecValueType;

use getset::{Getters, MutGetters, Setters};
use orion_variate::vars::EnvDict;

/// AI执行器，支持函数调用的AI任务执行器
///
/// 这个结构体提供了简化的AI任务执行接口，复用orion_ai的FunctionRegistry
/// 来实现工具调用功能。
///
/// # 示例
///
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
        let response = exec_unit.execute_with_func(base_prompt).await.err_conv()?;

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
        AiExecUnitBuilder::new(EnvDict::from(vars.global().export()))
            .with_config_opt(self.config.clone())
            .with_role_opt(self.role.clone())
            .with_tools(self.tools.clone())
            .build()
            .err_conv()
            .want("create ai exec unit")
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
    use orion_ai::{types::ExecutionStatus, AiResult, GlobalFunctionRegistry};
    use orion_error::TestAssert;
    use orion_sec::load_sec_dict;

    use super::*;

    #[tokio::test]
    async fn test_basic_ai_execution() -> AiResult<()> {
        GlobalFunctionRegistry::initialize().assert();
        let ai_builder = AiExecUnitBuilder::new(load_sec_dict().unwrap());
        let ai_exec = ai_builder
            .clone()
            .with_role("developer")
            //.with_tools(vec!["git-status".to_string()])
            .build()
            .want("create ai exec unit")?;

        let response = ai_exec
            //.execute_with_func("请检查当前Git仓库的状态，看看有哪些文件被修改了")
            .execute("请回答：1+1=?")
            .await?;
        match response.status {
            ExecutionStatus::Success => {
                println!("✅  {} ", response.content);
                for call in response.tool_calls {
                    println!("✅  {:#} ", call.result);
                }
            }
            _ => {
                eprintln!("❌ {}", response.content);
                panic!("false");
            }
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_executor_with_tools() -> AiResult<()> {
        GlobalFunctionRegistry::initialize().assert();
        let ai_builder = AiExecUnitBuilder::new(load_sec_dict().unwrap());
        let ai_exec = ai_builder
            .clone()
            .with_role("developer")
            .with_tools(vec!["git-status".to_string()])
            .build()
            .want("create ai exec unit")?;

        //let mut executor = AiExecutor::default();

        let response = ai_exec
            .execute_with_func("请检查当前Git仓库的状态，看看有哪些文件被修改了")
            .await?;

        match response.status {
            ExecutionStatus::Success => {
                println!("✅  {} ", response.content);
                for call in response.tool_calls {
                    println!("✅  {:#} ", call.result);
                }
            }
            _ => {
                eprintln!("❌ {}", response.content);
                panic!("false");
            }
        }
        Ok(())
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
