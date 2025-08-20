use log::info;
use orion_common::serde::Yamlable;
use orion_error::{ErrorConv, ErrorWith, ToStructError, UvsConfFrom};

use crate::config::roles::manager::RoleConfigManager;
use crate::error::{AiErrReason, AiError, AiResult};
use std::path::PathBuf;

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
        let manager = RoleConfigManager::from_yml(&PathBuf::from(path)).unwrap();
        Ok(manager)
    }

    /// 分层加载角色配置管理器
    /// 优先级：项目级配置 > 用户级配置
    pub fn layered_load(role_file: Option<PathBuf>) -> AiResult<RoleConfigManager> {
        let project_roles_path = role_file.unwrap_or(PathBuf::from("_gal/ai-roles.yml"));

        // 检查用户级配置路径
        let user_home = dirs::home_dir().ok_or_else(|| {
            AiError::from(AiErrReason::from_conf("无法获取用户主目录".to_string()))
        })?;
        let user_roles_path = user_home.join(".galaxy/ai-roles.yml");

        // 优先使用项目级配置
        if project_roles_path.exists() {
            println!(
                "Loading project-level roles configuration from {}...",
                project_roles_path.display()
            );
            let manager = RoleConfigManager::from_yml(&project_roles_path).err_conv()?;
            for k in manager.roles().keys() {
                info!("load role :{k}");
            }
            return Ok(manager);
        }

        // 其次使用用户级配置
        if user_roles_path.exists() {
            println!(
                "Loading user-level roles configuration from {}...",
                user_roles_path.display()
            );
            let manager = RoleConfigManager::from_yml(&user_roles_path).err_conv()?;
            return Ok(manager);
        }

        Err(AiError::from(AiErrReason::from_conf(
            "未找到有效的角色配置文件".to_string(),
        )))
        .with(&project_roles_path)
        .with(&user_roles_path)
    }

    /// 获取分层规则配置路径
    /// 优先级：项目级配置 > 用户级配置
    pub fn get_layered_rules_path(base_rules_path: &str) -> AiResult<PathBuf> {
        // 检查项目级规则配置
        let project_rules_path = PathBuf::from("_gal");
        if project_rules_path.exists() {
            return Ok(project_rules_path.join(base_rules_path));
        }

        // 检查用户级规则配置
        let user_home = dirs::home_dir().ok_or_else(|| {
            AiError::from(AiErrReason::from_conf("无法获取用户主目录".to_string()))
        })?;
        let user_rules_path = user_home.join(".galaxy");
        if user_rules_path.exists() {
            return Ok(user_rules_path.join(base_rules_path));
        }

        // 如果都没有找到，返回原始路径
        Ok(PathBuf::from(base_rules_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_load_with_valid_config() {
        // 创建临时配置文件
        let temp_dir = tempdir().unwrap();
        let config_path = temp_dir.path().join("test-roles.yml");

        let config_content = r#"
default_role:
  id: developer
default_model: deepseek-chat
roles:
  developer:
    name: developer
    description: 测试开发者角色
    system_prompt: 你是一个测试开发者
    used_model: deepseek-chat
    default_model: null
    rules_path: ai-rules/developer
  operations:
    name: operations
    description: 测试运维角色
    system_prompt: 你是一个测试运维专家
    used_model: deepseek-chat
    default_model: null
    rules_path: ai-rules/operations
"#;

        fs::write(&config_path, config_content).unwrap();

        // 测试加载配置
        let result = RoleConfigLoader::load(Some(config_path.to_string_lossy().to_string()));

        // 验证结果
        assert!(result.is_ok());
        let manager = result.unwrap();
        assert_eq!(manager.default_role().to_string(), "developer");
        assert!(manager.role_exists("developer"));
        assert!(manager.role_exists("operations"));

        // 验证角色配置内容
        let dev_config = manager.get_role_config("developer").unwrap();
        assert_eq!(dev_config.name(), "developer");
        assert_eq!(dev_config.description(), "测试开发者角色");
        assert_eq!(dev_config.system_prompt(), "你是一个测试开发者");
    }

    #[test]
    fn test_load_with_none_config() {
        let result = RoleConfigLoader::load(None);

        // 验证返回错误
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(error.to_string().contains("role config is none"));
    }

    #[test]
    fn test_get_layered_rules_path() {
        // 测试项目级规则路径
        let result = RoleConfigLoader::get_layered_rules_path("ai-rules/developer");
        assert!(result.is_ok());
        let path = result.unwrap();

        // 检查路径是否正确构建
        // 由于用户目录可能存在，我们应该检查返回的路径是否是有效的组合路径
        // 而不是硬编码比较特定路径
        assert!(path.ends_with("ai-rules/developer"));

        // 检查路径是否是项目级或用户级的组合
        let path_str = path.to_string_lossy();
        let is_project_path = path_str.contains("_gal");
        let is_user_path = path_str.contains(".galaxy");

        // 应该是项目级或用户级路径之一
        assert!(is_project_path || is_user_path);
    }
}
