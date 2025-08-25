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
