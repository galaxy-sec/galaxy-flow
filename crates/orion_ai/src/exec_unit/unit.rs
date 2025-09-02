use crate::{
    client::AiClientTrait, func::registry::FunctionRegistry, types::result::ExecutionResult,
    AiClient, AiResult, AiRoleID, FunctionResult,
};
use getset::{Getters, MutGetters, Setters, WithSetters};

/// AI执行单元，封装AI执行所需的核心组件
///
/// 这个结构体将AI客户端、角色和函数注册表组合成一个统一的执行单元，
/// 提供简洁的接口来执行AI任务。
///
/// # 示例
///
/// ```rust
/// use orion_ai::{AiExecUnit, AiClient, AiRoleID, FunctionRegistry};
///
/// // 假设已经创建了 client, role, registry
/// let exec_unit = AiExecUnit::new(client, role, registry);
/// let response = exec_unit.execute("你好，请介绍一下自己").await?;
/// ```

#[derive(Getters, MutGetters, Setters, WithSetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
pub struct AiExecUnit {
    client: AiClient,
    role: AiRoleID,
    registry: FunctionRegistry,
}

impl std::fmt::Debug for AiExecUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AiExecUnit")
            .field("role", &self.role)
            .field(
                "registry",
                &format!("FunctionRegistry({})", self.registry.get_functions().len()),
            )
            .field("client", &"AiClient".to_string())
            .finish()
    }
}

impl AiExecUnit {
    /// 创建新的执行单元
    ///
    /// # 参数
    ///
    /// * `client` - AI客户端实例
    /// * `role` - AI角色标识
    /// * `registry` - 函数注册表
    pub fn new(client: AiClient, role: AiRoleID, registry: FunctionRegistry) -> Self {
        Self {
            client,
            role,
            registry,
        }
    }

    pub async fn execute(&self, prompt: &str) -> AiResult<ExecutionResult> {
        let response = self.client.smart_role_request(&self.role, prompt).await?;

        // 将 AiResponse 转换为 ExecutionResult
        let tool_results = if let Some(tool_calls) = &response.tool_calls {
            tool_calls
                .iter()
                .map(|tool_call| {
                    FunctionResult {
                        name: tool_call.function.name.clone(),
                        result: serde_json::Value::Null, // 工具调用结果需要后续处理
                        error: None,
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(ExecutionResult::new(response.content).with_tool_calls(tool_results))
    }
    pub async fn execute_with_func(&self, prompt: &str) -> AiResult<ExecutionResult> {
        let response = self
            .client
            .role_funs_request(&self.role, prompt, self.registry().clone_functions())
            .await?;

        // 立即执行模式 (方案1): 检测到工具调用后立即执行
        // 注意：这是一个同步执行实现，AI会基于工具结果结束第一轮对话
        // TODO: 未来可扩展为支持多轮工具调用的链式执行模式
        let tool_results = if let Some(tool_calls) = &response.tool_calls {
            let mut results = Vec::new();

            for tool_call in tool_calls {
                // 使用函数注册表实际执行工具调用
                let execution_result = self.registry.execute_function(tool_call).await;

                match execution_result {
                    Ok(result) => {
                        results.push(FunctionResult {
                            name: tool_call.function.name.clone(),
                            result: result.result, // 实际执行结果
                            error: None,
                        });
                    }
                    Err(e) => {
                        results.push(FunctionResult {
                            name: tool_call.function.name.clone(),
                            result: serde_json::Value::Null,
                            error: Some(e.to_string()), // 记录错误信息
                        });
                    }
                }
            }
            results
        } else {
            Vec::new()
        };

        Ok(ExecutionResult::new(response.content).with_tool_calls(tool_results))
    }

    /// 消费执行单元，返回其组件
    ///
    /// # 返回
    ///
    /// 返回包含客户端、角色和函数注册表的元组
    pub fn into_components(self) -> (AiClient, AiRoleID, FunctionRegistry) {
        (self.client, self.role, self.registry)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{client::AiClientBuilder, config::AiConfig};

    #[tokio::test]
    async fn test_exec_unit_creation() {
        // 这个测试需要有效的AI配置，在实际环境中可能无法运行
        // 主要是测试编译和基本功能

        // 创建一个模拟的执行单元（在实际测试中需要有效的配置）
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role.clone(), registry);

        assert_eq!(exec_unit.role(), &role);
    }

    #[test]
    fn test_into_components() {
        // 测试into_components方法
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role.clone(), registry);
        let (_returned_client, returned_role, _returned_registry) = exec_unit.into_components();

        // 验证返回的组件与原始组件相同
        assert_eq!(returned_role, role);
        // client 和 registry 的比较需要特殊的比较逻辑
    }

    #[test]
    fn test_with_registry() {
        // 测试with_registry方法
        let config = AiConfig::example();
        let client = AiClientBuilder::new(config).build().unwrap();
        let role = client.roles().default_role().clone();
        let registry1 = FunctionRegistry::new();
        let registry2 = FunctionRegistry::new();

        let exec_unit = AiExecUnit::new(client, role, registry1);
        let updated_unit = exec_unit.with_registry(registry2.clone());

        // 验证更新后的注册表
        assert_eq!(
            updated_unit.registry().get_functions().len(),
            registry2.get_functions().len()
        );
    }
}
