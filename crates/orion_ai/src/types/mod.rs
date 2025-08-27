//! 类型定义模块
//!
//! 这个模块包含了orion_ai的统一类型定义，提供简化的结果类型。

// 模块导出
pub mod result;

// 重新导出主要类型，便于使用
pub use result::{ExecutionResult, ExecutionResultBuilder, ExecutionStatus};

/// 预导入的常用类型和trait
pub mod prelude {
    pub use super::{ExecutionResult, ExecutionResultBuilder, ExecutionStatus};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_types_module_imports() {
        // 测试模块导入是否正常
        let _result = ExecutionResult::new("test".to_string());
        let _builder = ExecutionResultBuilder::new();
    }
}
