use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use crate::ai::error::{AiError, AiErrReason, AiResult};

/// 角色配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 推荐模型
    pub recommended_model: String,
    /// 推荐模型列表
    pub recommended_models: Vec<String>,
}

/// 角色配置管理器
#[derive(Debug)]
pub struct RoleConfigManager {
    /// 角色配置映射
    roles: HashMap<String, RoleConfig>,
    /// 配置文件路径
    config_path: String,
}

impl RoleConfigManager {
    /// 创建新的角色配置管理器
    pub fn new(config_path: String) -> Self {
        Self {
            roles: HashMap::new(),
            config_path,
        }
    }

    /// 从文件加载角色配置
    pub fn load_config(&mut self) -> AiResult<()> {
        let path = Path::new(&self.config_path);
        
        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(
                format!("角色配置文件不存在: {}", self.config_path)
            )));
        }

        let content = fs::read_to_string(path)
            .map_err(|e| AiError::from(AiErrReason::ConfigError(
                format!("读取角色配置文件失败: {e}")
            )))?;

        let roles_config: HashMap<String, RoleConfig> = serde_json::from_str(&content)
            .map_err(|e| AiError::from(AiErrReason::ConfigError(
                format!("解析角色配置文件失败: {e}")
            )))?;

        self.roles = roles_config;
        Ok(())
    }

    /// 获取角色配置
    pub fn get_role_config(&self, role_key: &str) -> Option<&RoleConfig> {
        self.roles.get(role_key)
    }

    /// 获取所有可用的角色
    pub fn get_available_roles(&self) -> Vec<&String> {
        self.roles.keys().collect()
    }

    /// 重新加载配置
    pub fn reload_config(&mut self) -> AiResult<()> {
        self.roles.clear();
        self.load_config()
    }

    /// 检查角色是否存在
    pub fn role_exists(&self, role_key: &str) -> bool {
        self.roles.contains_key(role_key)
    }
}

impl Default for RoleConfigManager {
    fn default() -> Self {
        Self::new("src/ai/config/roles.json".to_string())
    }
}

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::io::Write;

    #[test]
    fn test_role_config_manager() {
        let test_config = r#"{
            "test_role": {
                "name": "测试角色",
                "description": "这是一个测试角色",
                "system_prompt": "你是一个测试角色",
                "recommended_model": "test-model",
                "recommended_models": ["test-model", "backup-model"]
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path);
        assert!(manager.load_config().is_ok());

        let role_config = manager.get_role_config("test_role");
        assert!(role_config.is_some());
        
        let config = role_config.unwrap();
        assert_eq!(config.name, "测试角色");
        assert_eq!(config.description, "这是一个测试角色");
        assert_eq!(config.system_prompt, "你是一个测试角色");
        assert_eq!(config.recommended_model, "test-model");
        assert_eq!(config.recommended_models, vec!["test-model", "backup-model"]);
    }

    #[test]
    fn test_role_config_loader() {
        let test_config = r#"{
            "developer": {
                "name": "开发者",
                "description": "专注于代码开发",
                "system_prompt": "你是一个开发者",
                "recommended_model": "dev-model",
                "recommended_models": ["dev-model"]
            }
        }"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.role_exists("developer"));
    }
}