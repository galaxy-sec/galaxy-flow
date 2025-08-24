use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{AiErrReason, AiResult, FunctionCall, FunctionDefinition, FunctionResult};
use orion_error::{ToStructError, UvsLogicFrom};

/// 简化的函数执行器 trait
#[async_trait]
pub trait FunctionExecutor: Send + Sync {
    /// 执行函数调用
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult>;

    /// 获取支持的函数列表
    fn supported_functions(&self) -> Vec<String>;

    /// 获取函数schema
    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition>;
}

/// 简化的函数注册表
#[derive(Default)]
pub struct FunctionRegistry {
    functions: HashMap<String, FunctionDefinition>,
    executors: HashMap<String, Arc<dyn FunctionExecutor>>,
}

impl FunctionRegistry {
    /// 创建新的注册表
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册函数
    pub fn register_function(&mut self, function: FunctionDefinition) -> AiResult<()> {
        self.functions.insert(function.name.clone(), function);
        Ok(())
    }

    /// 注册执行器
    pub fn register_executor(
        &mut self,
        name: String,
        executor: Arc<dyn FunctionExecutor>,
    ) -> AiResult<()> {
        self.executors.insert(name, executor);
        Ok(())
    }

    /// 获取所有函数定义
    pub fn get_functions(&self) -> Vec<&FunctionDefinition> {
        self.functions.values().collect()
    }

    /// 根据名称获取函数定义
    pub fn get_function(&self, name: &str) -> Option<&FunctionDefinition> {
        self.functions.get(name)
    }

    /// 执行函数调用
    pub async fn execute_function(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        let executor = self
            .executors
            .get(&function_call.function.name)
            .ok_or_else(|| AiErrReason::from_logic("TODO: executor not found".into()).to_err())?;

        executor.execute(function_call).await
    }

    /// 检查是否支持指定函数
    pub fn supports_function(&self, function_name: &str) -> bool {
        self.executors.contains_key(function_name)
    }

    /// 获取所有支持的函数名称
    pub fn get_supported_function_names(&self) -> Vec<String> {
        self.executors.keys().cloned().collect()
    }

    /// 批量注册函数
    pub fn register_functions(&mut self, functions: Vec<FunctionDefinition>) -> AiResult<()> {
        for function in functions {
            self.register_function(function)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // 测试用的模拟执行器
    struct MockExecutor {
        name: String,
    }

    impl MockExecutor {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
            }
        }
    }

    #[async_trait]
    impl FunctionExecutor for MockExecutor {
        async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
            // 解析 JSON 字符串参数
            let args: serde_json::Value = serde_json::from_str(&function_call.function.arguments)
                .unwrap_or_else(|_| serde_json::json!({}));

            Ok(FunctionResult {
                name: function_call.function.name.clone(),
                result: json!({
                    "message": format!("Mock execution of {}", function_call.function.name),
                    "args": args
                }),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            vec![self.name.clone()]
        }

        fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
            if function_name == self.name {
                Some(FunctionDefinition {
                    name: self.name.clone(),
                    description: format!("Mock function {}", self.name),
                    parameters: vec![],
                })
            } else {
                None
            }
        }
    }

    #[tokio::test]
    async fn test_function_registry() {
        let mut registry = FunctionRegistry::new();

        // 测试函数注册
        let test_function = FunctionDefinition {
            name: "test_function".to_string(),
            description: "Test function".to_string(),
            parameters: vec![],
        };

        assert!(registry.register_function(test_function.clone()).is_ok());
        assert_eq!(registry.get_functions().len(), 1);
        assert_eq!(registry.get_function("test_function"), Some(&test_function));
    }

    #[tokio::test]
    async fn test_executor_registration() {
        let mut registry = FunctionRegistry::new();
        let executor = Arc::new(MockExecutor::new("test_exec"));

        assert!(registry
            .register_executor("test_exec".to_string(), executor.clone())
            .is_ok());
        assert!(registry.supports_function("test_exec"));
        assert!(!registry.supports_function("unknown"));
    }

    #[tokio::test]
    async fn test_function_execution() {
        let mut registry = FunctionRegistry::new();
        let executor = Arc::new(MockExecutor::new("test_exec"));

        registry
            .register_executor("test_exec".to_string(), executor)
            .unwrap();

        let function_call = FunctionCall {
            index: Some(0),
            id: "call_test_001".to_string(),
            r#type: "function".to_string(),
            function: crate::provider::FunctionCallInfo {
                name: "test_exec".to_string(),
                arguments: "{\"param1\":\"value1\"}".to_string(),
            },
        };

        let result = registry.execute_function(&function_call).await;
        assert!(result.is_ok());

        let result = result.unwrap();
        assert_eq!(result.name, "test_exec");
        assert!(result.result.is_object());
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_execution_not_found() {
        let registry = FunctionRegistry::new();

        let function_call = FunctionCall {
            index: Some(0),
            id: "call_test_002".to_string(),
            r#type: "function".to_string(),
            function: crate::provider::FunctionCallInfo {
                name: "unknown_function".to_string(),
                arguments: "{}".to_string(),
            },
        };

        let result = registry.execute_function(&function_call).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_function_registration() {
        let mut registry = FunctionRegistry::new();

        let functions = vec![
            FunctionDefinition {
                name: "func1".to_string(),
                description: "Function 1".to_string(),
                parameters: vec![],
            },
            FunctionDefinition {
                name: "func2".to_string(),
                description: "Function 2".to_string(),
                parameters: vec![],
            },
        ];

        assert!(registry.register_functions(functions).is_ok());
        assert_eq!(registry.get_functions().len(), 2);
    }
}
