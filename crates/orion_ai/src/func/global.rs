use std::sync::{Arc, OnceLock};

use crate::{func::registry::FunctionRegistry, FunctionExecutor};

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
            #[allow(static_mut_refs)]
            if INSTANCE.is_none() {
                INSTANCE = Some(GlobalFunctionRegistry {
                    initialized_registry: OnceLock::new(),
                });
            }
            #[allow(static_mut_refs)]
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

    /// 🎯 获取注册表的克隆副本，并根据指定工具列表进行过滤
    pub fn get_registry_with_tools(
        tools: &[String],
    ) -> Result<FunctionRegistry, orion_error::UvsReason> {
        // 首先获取完整的注册表副本
        let full_registry = Self::get_registry()?;

        // 如果工具列表为空，返回所有函数
        if tools.is_empty() {
            return Ok(full_registry);
        }

        // 否则，过滤出指定工具的函数
        let filtered_functions = full_registry
            .get_functions()
            .into_iter()
            .filter(|func_def| tools.contains(&func_def.name))
            .cloned()
            .collect::<Vec<_>>();

        // 创建新的注册表并只包含过滤后的函数
        let mut filtered_registry = FunctionRegistry::new();
        for function_def in filtered_functions {
            filtered_registry
                .register_function(function_def)
                .map_err(|e| {
                    orion_error::UvsReason::validation_error(format!(
                        "Failed to register filtered function: {}",
                        e
                    ))
                })?;
        }

        // 复制执行器引用
        for tool_name in tools {
            if let Some(executor) = full_registry.get_executor(tool_name) {
                filtered_registry
                    .register_executor(tool_name.clone(), executor)
                    .map_err(|e| {
                        orion_error::UvsReason::validation_error(format!(
                            "Failed to register executor for {}: {}",
                            tool_name, e
                        ))
                    })?;
            }
        }

        Ok(filtered_registry)
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
    // 添加测试用例来验证 get_registry_with_tools 功能
    #[tokio::test]
    async fn test_get_registry_with_tools() {
        // 重置并初始化注册表
        GlobalFunctionRegistry::reset();
        assert!(GlobalFunctionRegistry::initialize().is_ok());

        // 测试指定工具列表
        let tools = vec!["git-status".to_string(), "git-add".to_string()];
        let filtered_registry = GlobalFunctionRegistry::get_registry_with_tools(&tools).unwrap();

        let filtered_functions = filtered_registry.get_supported_function_names();
        assert_eq!(filtered_functions.len(), 2);
        assert!(filtered_functions.contains(&"git-status".to_string()));
        assert!(filtered_functions.contains(&"git-add".to_string()));
        assert!(!filtered_functions.contains(&"git-commit".to_string()));

        // 测试单个工具
        let single_tool = vec!["git-status".to_string()];
        let single_registry =
            GlobalFunctionRegistry::get_registry_with_tools(&single_tool).unwrap();

        let single_functions = single_registry.get_supported_function_names();
        assert_eq!(single_functions.len(), 1);
        assert!(single_functions.contains(&"git-status".to_string()));

        // 测试空工具列表（应该返回所有工具）
        let empty_tools: Vec<String> = vec![];
        let full_registry = GlobalFunctionRegistry::get_registry_with_tools(&empty_tools).unwrap();

        let full_functions = full_registry.get_supported_function_names();
        let all_registry = GlobalFunctionRegistry::get_registry().unwrap();
        let all_functions = all_registry.get_supported_function_names();

        assert_eq!(full_functions.len(), all_functions.len());
        for func_name in &full_functions {
            assert!(all_functions.contains(func_name));
        }

        // 测试不存在的工具
        let nonexistent_tools = vec!["nonexistent_tool".to_string()];
        let empty_registry =
            GlobalFunctionRegistry::get_registry_with_tools(&nonexistent_tools).unwrap();

        let empty_functions = empty_registry.get_supported_function_names();
        assert_eq!(empty_functions.len(), 0);

        // 测试混合存在的和不存在的工具
        let mixed_tools = vec!["git-status".to_string(), "nonexistent_tool".to_string()];
        let mixed_registry = GlobalFunctionRegistry::get_registry_with_tools(&mixed_tools).unwrap();

        let mixed_functions = mixed_registry.get_supported_function_names();
        assert_eq!(mixed_functions.len(), 1);
        assert!(mixed_functions.contains(&"git-status".to_string()));
    }
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
        assert!(function_names.contains(&"git-status".to_string()));
        assert!(function_names.contains(&"git-commit".to_string()));
        assert!(function_names.contains(&"git-add".to_string()));
        assert!(function_names.contains(&"git-push".to_string()));
        assert!(function_names.contains(&"git-diff".to_string()));
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
