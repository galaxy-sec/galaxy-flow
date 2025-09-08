use crate::ability::ai::AI_CALL_RESULT;
use crate::ability::ai::AI_CALL_VALUE;
use crate::ability::gxl::do_gxl_run;
use crate::ability::prelude::*;
use crate::cmd::GxlCmd;
use crate::util::OptionFrom;
use async_trait::async_trait;
use orion_sec::sec::NoSecConv;
use orion_sec::sec::SecFrom;
use orion_sec::sec::SecValueType;
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
        let cmd = self.exe_cmd().get().cloned().expect("exe_cmd not exists");
        let cmd = cmd.with_flows(self.flow().clone());
        let vars = self.exe_vars().get().cloned().expect("exe_vars not exists");
        let task_value = do_gxl_run(cmd, &vars, true, None)
            .await
            .owe_net()?;

        if let (Some(call_result), Some(call_value)) = (
            task_value.vars.get(AI_CALL_RESULT),
            task_value.vars.get(AI_CALL_VALUE),
        )
            && call_result == SecValueType::sec_from(true) {
                return Ok(FunctionResult {
                    name: function_call.function.name.clone(),
                    result: serde_json::json!(call_value.no_sec()),
                    error: None,
                });
        }
        Ok(FunctionResult {
            name: function_call.function.name.clone(),
            result: serde_json::json!(false),
            error: "call no response".to_opt(),
        })
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


}
