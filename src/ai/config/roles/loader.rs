use orion_error::{ToStructError, UvsConfFrom};

use crate::ai::config::roles::manager::RoleConfigManager;
use crate::ai::error::{AiErrReason, AiError, AiResult};
use std::path::{Path, PathBuf};

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
        let path = config_path
            .ok_or_else(|| AiErrReason::from_conf("role config is none".to_string()).to_err())?;
        let mut manager = RoleConfigManager::new(path);
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
            println!("📄 Loading user-level roles configuration from {user_roles_str}...");
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
    pub fn get_layered_rules_path(base_rules_path: &str) -> AiResult<PathBuf> {
        // 检查项目级规则配置
        let project_rules_path = PathBuf::from("_gal/ai-rules");
        if project_rules_path.exists() {
            return Ok(project_rules_path.join(base_rules_path));
        }

        // 检查用户级规则配置
        let user_home = dirs::home_dir().ok_or_else(|| {
            AiError::from(AiErrReason::ConfigError("无法获取用户主目录".to_string()))
        })?;
        let user_rules_path = user_home.join(".galaxy/ai-rules");
        if user_rules_path.exists() {
            return Ok(user_rules_path.join(base_rules_path));
        }

        // 如果都没有找到，返回原始路径
        Ok(PathBuf::from(base_rules_path))
    }
}
