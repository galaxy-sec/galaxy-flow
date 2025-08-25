use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

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
#[derive(Clone, Default)]
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

    /// 新增：克隆注册表
    pub fn clone_registry(&self) -> Self {
        let mut new_registry = Self::new();

        // 克隆函数定义
        for (name, function) in &self.functions {
            new_registry
                .functions
                .insert(name.clone(), function.clone());
        }

        // 克隆执行器引用（Arc 可以安全克隆）
        for (name, executor) in &self.executors {
            new_registry
                .executors
                .insert(name.clone(), executor.clone());
        }

        new_registry
    }

    /// 新增：获取所有函数定义的克隆
    pub fn clone_functions(&self) -> Vec<FunctionDefinition> {
        self.functions.values().cloned().collect()
    }
}

/// 全局函数注册表管理器
pub struct GlobalFunctionRegistry {
    // 使用 OnceLock 确保只初始化一次
    initialized_registry: OnceLock<Arc<FunctionRegistry>>,
}

impl GlobalFunctionRegistry {
    /// 获取全局单例
    pub fn instance() -> &'static Self {
        static mut INSTANCE: Option<GlobalFunctionRegistry> = None;
        unsafe {
            if INSTANCE.is_none() {
                INSTANCE = Some(GlobalFunctionRegistry {
                    initialized_registry: OnceLock::new(),
                });
            }
            INSTANCE.as_ref().unwrap()
        }
    }

    /// 初始化并注册所有工具（应用启动时调用）
    pub fn initialize() -> Result<(), orion_error::UvsReason> {
        let instance = Self::instance();

        // 确保只初始化一次
        if instance.initialized_registry.get().is_some() {
            return Ok(());
        }

        // 创建并注册所有工具
        let registry = Arc::new(Self::create_and_register_tools()?);
        let _ = instance.initialized_registry.set(registry);

        Ok(())
    }

    /// 创建注册表并注册所有工具（硬编码）
    fn create_and_register_tools() -> Result<FunctionRegistry, orion_error::UvsReason> {
        let mut registry = FunctionRegistry::new();

        // 硬编码注册 Git 工具
        Self::register_git_tools(&mut registry)?;

        Ok(registry)
    }

    /// 显式注册 Git 工具
    fn register_git_tools(registry: &mut FunctionRegistry) -> Result<(), orion_error::UvsReason> {
        use crate::func::git::{create_git_functions, GitFunctionExecutor};
        use std::sync::Arc;

        // 注册函数定义
        let git_functions = create_git_functions();
        for function in git_functions {
            registry.register_function(function).map_err(|e| {
                orion_error::UvsReason::validation_error(format!(
                    "Failed to register git function: {}",
                    e
                ))
            })?;
        }

        // 注册执行器
        let git_executor = Arc::new(GitFunctionExecutor);
        for function_name in git_executor.supported_functions() {
            registry
                .register_executor(function_name, git_executor.clone())
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register git executor: {}",
                        e
                    ))
                })?;
        }

        Ok(())
    }

    /// 获取注册表的克隆副本（避免锁竞争）
    pub fn get_registry() -> Result<FunctionRegistry, orion_error::UvsReason> {
        let instance = Self::instance();

        // 获取已初始化的注册表
        let global_registry = instance.initialized_registry.get().ok_or_else(|| {
            orion_error::UvsReason::validation_error(
                "Global function registry not initialized. Call initialize() first.",
            )
        })?;

        // 返回克隆副本，避免锁竞争
        Ok(global_registry.as_ref().clone_registry())
    }

    /// 重置注册表（主要用于测试）
    pub fn reset() {
        unsafe {
            let instance = Self::instance();
            let ptr = instance as *const GlobalFunctionRegistry as *mut GlobalFunctionRegistry;
            (*ptr).initialized_registry = OnceLock::new();
        }
    }
}

#[cfg(test)]
mod global_registry_tests {
    use super::*;

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

    #[tokio::test]
    async fn test_registry_cloning() {
        // 初始化全局注册表
        GlobalFunctionRegistry::reset();
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 获取第一个副本
        let registry1 = GlobalFunctionRegistry::get_registry().unwrap();
        let function_names1 = registry1.get_supported_function_names();

        // 获取第二个副本
        let registry2 = GlobalFunctionRegistry::get_registry().unwrap();
        let function_names2 = registry2.get_supported_function_names();

        // 验证两个副本包含相同的函数
        assert_eq!(function_names1, function_names2);

        // 验证两个副本都可以正常工作
        for function_name in &function_names1 {
            assert!(registry1.supports_function(function_name));
            assert!(registry2.supports_function(function_name));
        }
    }

    #[tokio::test]
    async fn test_double_initialization() {
        GlobalFunctionRegistry::reset();

        // 第一次初始化
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 第二次初始化应该不会失败
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 注册表应该仍然可用
        let registry = GlobalFunctionRegistry::get_registry();
        assert!(registry.is_ok());
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
