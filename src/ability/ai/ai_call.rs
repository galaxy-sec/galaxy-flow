use crate::ability::prelude::*;
use crate::ability::GxCmd;
use crate::ability::GxRun;
use crate::cmd::GxlCmd;
use crate::execution::runnable::AsyncRunnableWithSenderTrait;
use crate::util::OptionFrom;
use async_trait::async_trait;
use std::sync::Arc;
use std::sync::OnceLock;

use orion_ai::{
    AiConfig, AiExecUnit, AiExecUnitBuilder, AiResult, ExecutionResult, FunctionCall,
    FunctionDefinition, FunctionExecutor, FunctionResult, GlobalFunctionRegistry,
};
use orion_error::ErrorConv;

use getset::{Getters, MutGetters, Setters};
use orion_variate::vars::EnvDict;

/// AI执行器，支持函数调用的AI任务执行器
///
/// 这个结构体提供了简化的AI任务执行接口，复用orion_ai的FunctionRegistry
/// 来实现工具调用功能。
///
/// # 示例
///
#[derive(Debug, Getters, MutGetters, Setters, Clone)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiGxlCall {
    key: String,
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    flow: String,
    exe_unit: OnceLock<Arc<AiExecUnit>>,
    exe_vars: OnceLock<VarSpace>,
    exe_cmd: OnceLock<GxlCmd>,
}

impl Default for AiGxlCall {
    fn default() -> Self {
        Self {
            key: String::new(),
            role: None,
            task: None,
            config: None,
            flow: "unknow".to_string(),
            exe_unit: OnceLock::new(),
            exe_vars: OnceLock::new(),
            exe_cmd: OnceLock::new(),
        }
    }
}

impl AiGxlCall {
    pub async fn execute_call(&self) -> AiResult<ExecutionResult> {
        if let Some(exec_unit) = self.exe_unit.get() {
            let task_prompt = self.task.as_deref().unwrap_or("请完成任务");
            return exec_unit.execute_with_func(task_prompt).await;
        } else {
            unreachable!("ai-exec_unit not initialized. Call setup_exec_unit first.")
        }
    }

    /// 创建执行单元
    ///
    /// 使用配置创建AI执行单元，封装客户端、角色和函数注册表。
    fn setup_exec_unit(&self, ctx: ExecContext, vars: &VarSpace) -> ExecResult<()> {
        // 使用构建器创建执行单元
        if self.exe_unit.get().is_none() {
            let exec_unit = Arc::new(
                AiExecUnitBuilder::new(EnvDict::from(vars.global().export()))
                    .with_config_opt(self.config.clone())
                    .with_role_opt(self.role.clone())
                    //.with_tools(self.flow.clone())
                    .build()
                    .err_conv()
                    .want("create ai exec unit")?,
            );
            self.exe_unit
                .set(exec_unit)
                .expect("OnceLock should not be already set");
        }
        if self.exe_vars.get().is_none() {
            self.exe_vars
                .set(vars.clone())
                .expect("OnceLock should not be already set");
        }
        if self.exe_cmd.get().is_none() {
            self.exe_cmd
                .set(ctx.gxl_cmd().as_ref().clone())
                .expect("OnceLock should not be already set");
        }
        Ok(())
    }
    fn call_key(&self) -> String {
        format!("gxl_{}", self.key)
    }
    fn call_define(&self) -> FunctionDefinition {
        FunctionDefinition {
            name: self.call_key(),
            description: self.task().clone().unwrap_or("gxl call".to_string()),
            parameters: vec![],
        }
    }
}

impl ComponentMeta for AiGxlCall {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("ai_fun")
    }
}
#[async_trait::async_trait]
impl FunctionExecutor for AiGxlCall {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let cmd = self.exe_cmd().get().clone().expect("exe_cmd not exists");
        let cmd = cmd.with_flows(vec![self.flow().clone()]);
        let vars = self.exe_vars().get().cloned().expect("exe_vars not exists");
        //let run_path = PathBuf::from(exp.eval(&self.run_path)?);
        do_gxl_run(run_path, cmd, &vars, true, None).await?;

        let gxl = GxRun::new("./", "./", "env", vec![self.flow.clone()], true);
        let result = gxl
            .async_exec(
                ExecContext::default(),
                self.exe_vars.get().cloned().unwrap_or(VarSpace::default()),
                None,
            )
            .await;
        //let response = self.execute_call().await?;
        return Ok(if response.tool_calls.is_empty() {
            FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!(false),
                error: "call no response".to_opt(),
            }
        } else {
            response.tool_calls[0].clone()
        });
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![self.call_key()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        if function_name == self.call_key() {
            Some(self.call_define())
        } else {
            None
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for AiGxlCall {
    async fn async_exec(&self, ctx: ExecContext, vars: VarSpace) -> TaskResult {
        let fun_key = self.call_key();
        self.setup_exec_unit(ctx, &vars)?;
        GlobalFunctionRegistry::register_function(self.call_define()).err_conv()?;
        GlobalFunctionRegistry::register_executor(fun_key, Arc::new(self.clone())).err_conv()?;
        Ok(TaskValue::new(vars, ExecOut::Ignore))
    }
}

#[cfg(test)]
mod tests {
    use orion_ai::GlobalFunctionRegistry;
    use orion_error::TestAssert;

    use crate::infra::once_init_log;

    use super::*;

    #[tokio::test]
    async fn test_clone_and_debug() -> ExecResult<()> {
        once_init_log();
        GlobalFunctionRegistry::initialize().assert();
        let mut executor = AiGxlCall::default();
        executor.set_role(Some("developer".to_string()));
        executor.set_task(Some(
            "请检查当前Git仓库的状态，看看有哪些文件被修改了".to_string(),
        ));
        executor.set_tools(vec!["git-status".to_string()]);
        let _x = executor
            .async_exec(ExecContext::default(), VarSpace::sys_init()?)
            .await?;
        //println!("{:#}", x.vars.get(AI_CONTENT).assert());
        //println!("{:#}", x.vars.get(AI_CALL_RESULT).assert());
        //println!("{:#}", x.vars.get(AI_CALL_VALUE).assert());
        Ok(())
    }
}
