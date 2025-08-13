use crate::ai::error::{AiErrReason, AiError, AiResult};
use crate::ai::config::roles::manager::RoleConfigManager;
use std::path::Path;

/// 角色配置加载器
pub struct RoleConfigLoader;

impl Default for RoleConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleConfigLoader {
    /// 创建新的角色配置加载器
    pub fn new() -> Self {
        Self
    }

    /// 创建并加载角色配置管理器
    pub fn load(config_path: Option<String>) -> AiResult<RoleConfigManager> {
        let path = config_path.unwrap_or_else(|| "src/ai/config/roles.json".to_string());
        let mut manager = RoleConfigManager::new(path);
        manager.load_config()?;
        Ok(manager)
    }

    /// 创建默认角色配置管理器
    pub fn load_default() -> AiResult<RoleConfigManager> {
        Self::load(None)
    }

    /// 自动加载角色配置管理器（只使用roles.yaml）
    pub fn auto_load(
        _simplified_path: Option<String>,
        legacy_path: Option<String>,
    ) -> AiResult<RoleConfigManager> {
        let legacy = legacy_path.unwrap_or_else(|| "src/ai/config/roles.yaml".to_string());
        let mut manager = RoleConfigManager::new(legacy.clone());
        println!("📄 Loading roles configuration from {legacy}...");
        manager.load_config()?;
        Ok(manager)
    }

    /// 分层加载角色配置管理器
    /// 优先级：项目级配置 > 用户级配置
    pub fn layered_load() -> AiResult<RoleConfigManager> {
        // 检查项目级配置是否存在
        let project_roles_path = "_gal/ai-roles.yaml";
        let _project_rules_path = "_gal/ai-rules";

        // 检查用户级配置路径
        let user_home = dirs::home_dir().ok_or_else(|| {
            AiError::from(AiErrReason::ConfigError("无法获取用户主目录".to_string()))
        })?;
        let user_roles_path = user_home.join(".galaxy/ai-roles.yaml");
        let _user_rules_path = user_home.join(".galaxy/ai-rules");

        // 优先使用项目级配置
        if Path::new(project_roles_path).exists() {
            println!("📄 Loading project-level roles configuration from {project_roles_path}...");
            let mut manager = RoleConfigManager::new(project_roles_path.to_string());
            manager.load_config()?;
            return Ok(manager);
        }

        // 其次使用用户级配置
        if user_roles_path.exists() {
            let user_roles_str = user_roles_path.to_str().ok_or_else(|| {
                AiError::from(AiErrReason::ConfigError(
                    "用户级配置路径转换失败".to_string(),
                ))
            })?;
            println!(
                "📄 Loading user-level roles configuration from {}...",
                user_roles_str
            );
            let mut manager = RoleConfigManager::new(user_roles_str.to_string());
            manager.load_config()?;
            return Ok(manager);
        }

        Err(AiError::from(AiErrReason::ConfigError(
            "未找到有效的角色配置文件".to_string(),
        )))
    }

    /// 获取分层规则配置路径
    /// 优先级：项目级配置 > 用户级配置
    pub fn get_layered_rules_path(base_rules_path: &str) -> AiResult<String> {
        // 检查项目级规则配置
        let project_rules_path = "_gal/ai-rules";
        if Path::new(project_rules_path).exists() {
            return Ok(project_rules_path.to_string());
        }

        // 检查用户级规则配置
        let user_home = dirs::home_dir().ok_or_else(|| {
            AiError::from(AiErrReason::ConfigError("无法获取用户主目录".to_string()))
        })?;
        let user_rules_path = user_home.join(".galaxy/ai-rules");
        if user_rules_path.exists() {
            return Ok(user_rules_path
                .to_str()
                .ok_or_else(|| {
                    AiError::from(AiErrReason::ConfigError(
                        "用户级规则路径转换失败".to_string(),
                    ))
                })?
                .to_string());
        }

        // 如果都没有找到，返回原始路径
        Ok(base_rules_path.to_string())
    }
}