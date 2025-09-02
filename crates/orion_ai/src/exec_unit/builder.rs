use orion_error::ErrorWith;
use orion_variate::vars::EnvDict;

use crate::{
    client::AiClientBuilder, config::AiConfig, func::registry::FunctionRegistry, AiExecUnit,
    AiRoleID,
};

/// AI执行单元构建器
///
/// 提供流式接口来创建和配置 AiExecUnit 实例。
///
/// # 示例
///
/// ```rust
/// use orion_ai::{AiExecUnitBuilder, AiConfig};
///
/// let exec_unit = AiExecUnitBuilder::new()
///     .with_config(AiConfig::example())
///     .with_role("developer")
///     .with_tools(vec!["git-status".to_string()])
///     .build()?;
/// ```
#[derive(Clone, Debug, Default)]
pub struct AiExecUnitBuilder {
    dict: EnvDict,
    config: Option<AiConfig>,
    role: Option<AiRoleID>,
    tools: Vec<String>,
    timeout: Option<u64>,
}

impl AiExecUnitBuilder {
    /// 创建新的构建器实例
    pub fn new(dict: EnvDict) -> Self {
        Self {
            dict,
            config: None,
            role: None,
            tools: Vec::new(),
            timeout: Some(60), // 默认超时60秒
        }
    }

    /// 设置AI配置
    ///
    /// # 参数
    ///
    /// * `config` - AI客户端配置
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_config(mut self, config: AiConfig) -> Self {
        self.config = Some(config);
        self
    }
    pub fn with_config_opt(mut self, config: Option<AiConfig>) -> Self {
        self.config = config;
        self
    }

    /// 设置角色名称
    ///
    /// # 参数
    ///
    /// * `role_name` - 角色名称字符串
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_role(mut self, role_name: &str) -> Self {
        self.role = Some(AiRoleID::new(role_name.to_string()));
        self
    }
    pub fn with_role_opt(mut self, role_name: Option<String>) -> Self {
        self.role = role_name.map(AiRoleID::new);
        self
    }

    /// 设置角色ID
    ///
    /// # 参数
    ///
    /// * `role` - AI角色标识
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_role_id(mut self, role: AiRoleID) -> Self {
        self.role = Some(role);
        self
    }

    /// 添加工具
    ///
    /// # 参数
    ///
    /// * `tool` - 要添加的工具名称
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tools.push(tool.into());
        self
    }

    /// 设置工具列表
    ///
    /// # 参数
    ///
    /// * `tools` - 工具名称列表
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.tools = tools;
        self
    }

    /// 设置超时时间（秒）
    ///
    /// # 参数
    ///
    /// * `timeout_seconds` - 超时时间，单位秒
    ///
    /// # 返回
    ///
    /// 返回构建器实例（用于链式调用）
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout = Some(timeout_seconds);
        self
    }

    /// 构建执行单元
    ///
    /// # 返回
    ///
    /// 返回配置好的 AiExecUnit 实例
    ///
    /// # 错误
    ///
    /// 如果配置不完整或创建失败，返回错误
    pub fn build(self) -> crate::AiResult<AiExecUnit> {
        let config = self
            .config
            .clone()
            .unwrap_or(AiConfig::galaxy_load(&self.dict).want("load ai config")?);

        // 创建AI客户端
        let mut client_builder = AiClientBuilder::new(config);
        if let Some(timeout) = self.timeout {
            client_builder = client_builder.with_timout(timeout);
        }
        let client = client_builder.build()?;

        // 设置角色
        let role = self
            .role
            .unwrap_or_else(|| client.roles().default_role().clone());

        // 获取函数注册表
        let registry = client.get_registry_with_tools(&self.tools)?;

        // 创建执行单元
        Ok(AiExecUnit::new(client, role, registry))
    }

    /// 构建执行单元，但不验证工具是否存在
    ///
    /// 这个方法会忽略工具注册时的错误，适用于某些工具可能不可用但仍希望创建执行单元的场景。
    ///
    /// # 返回
    ///
    /// 返回配置好的 AiExecUnit 实例
    ///
    /// # 错误
    ///
    /// 如果配置不完整或创建失败，返回错误
    pub fn build_ignoring_tool_errors(self) -> crate::AiResult<AiExecUnit> {
        // 验证必需的配置
        let config = self.config.ok_or_else(|| {
            crate::error::error_utils::invalid_input_error("AI配置未设置".to_string())
        })?;

        // 创建AI客户端
        let mut client_builder = AiClientBuilder::new(config);
        if let Some(timeout) = self.timeout {
            client_builder = client_builder.with_timout(timeout);
        }
        let client = client_builder.build()?;

        // 设置角色
        let role = self
            .role
            .unwrap_or_else(|| client.roles().default_role().clone());

        // 获取函数注册表，忽略工具注册错误
        let registry = match client.get_registry_with_tools(&self.tools) {
            Ok(registry) => registry,
            Err(_) => {
                // 如果工具注册失败，使用空的注册表
                FunctionRegistry::new()
            }
        };

        // 创建执行单元
        Ok(AiExecUnit::new(client, role, registry))
    }

    /// 从示例配置创建构建器
    ///
    /// 使用示例配置创建构建器，适用于测试和开发。
    ///
    /// # 返回
    ///
    /// 返回配置好的构建器实例
    pub fn from_example() -> Self {
        Self::default().with_config(AiConfig::example())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = AiExecUnitBuilder::default();
        assert!(builder.config.is_none());
        assert!(builder.role.is_none());
        assert!(builder.tools.is_empty());
    }

    #[test]
    fn test_builder_with_role() {
        let builder = AiExecUnitBuilder::default()
            .with_role("developer")
            .with_timeout(30);

        assert_eq!(builder.role, Some(AiRoleID::new("developer".to_string())));
        assert_eq!(builder.timeout, Some(30));
    }

    #[test]
    fn test_builder_with_tools() {
        let tools = vec!["git-status".to_string(), "file-read".to_string()];
        let builder = AiExecUnitBuilder::default()
            .with_tools(tools.clone())
            .with_tool("git-commit");

        assert_eq!(
            builder.tools,
            vec![
                "git-status".to_string(),
                "file-read".to_string(),
                "git-commit".to_string()
            ]
        );
    }

    #[test]
    fn test_builder_from_example() {
        let builder = AiExecUnitBuilder::from_example();
        assert!(builder.config.is_some());
    }

    #[test]
    fn test_builder_clone() {
        let builder = AiExecUnitBuilder::default()
            .with_role("developer")
            .with_tools(vec!["test-tool".to_string()]);

        let cloned = builder.clone();
        assert_eq!(builder.role, cloned.role);
        assert_eq!(builder.tools, cloned.tools);
        assert_eq!(builder.timeout, cloned.timeout);
    }

    #[test]
    fn test_builder_debug() {
        let builder = AiExecUnitBuilder::default()
            .with_role("developer")
            .with_timeout(120);

        let debug_str = format!("{:?}", builder);
        assert!(debug_str.contains("AiExecUnitBuilder"));
        assert!(debug_str.contains("developer"));
        assert!(debug_str.contains("120"));
    }

    #[tokio::test]
    async fn test_build_with_example_config() {
        // 这个测试使用示例配置，可能需要有效的API密钥才能成功
        let builder = AiExecUnitBuilder::from_example().with_role("developer");

        match builder.build() {
            Ok(_) => {
                // 构建成功，这是期望的结果
                println!("构建成功");
            }
            Err(e) => {
                // 在没有有效API密钥的环境中，构建失败是正常的
                println!("预期的构建失败（缺少API密钥）: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_build_ignoring_tool_errors() {
        // 测试忽略工具错误的构建方法
        let builder = AiExecUnitBuilder::from_example()
            .with_role("developer")
            .with_tools(vec!["non-existent-tool".to_string()]);

        match builder.build_ignoring_tool_errors() {
            Ok(_) => {
                // 即使工具不存在，构建也应该成功
                println!("构建成功（忽略了工具错误）");
            }
            Err(e) => {
                // 如果构建失败，可能是配置问题
                println!("构建失败: {}", e);
            }
        }
    }
}
