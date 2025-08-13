use crate::ai::error::{AiErrReason, AiError, AiResult};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
    /// 规则配置文件路径
    pub rules_config: Option<String>,
}

/// 规则配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// 规则集合
    pub rules: Vec<String>,
}

/// 角色配置管理器
#[derive(Debug)]
pub struct RoleConfigManager {
    /// 角色配置映射
    pub roles: HashMap<String, RoleConfig>,
    /// 配置文件路径
    pub config_path: String,
}

impl RoleConfigManager {
    /// 保存角色配置到YAML文件
    pub fn save_config(&self) -> AiResult<()> {
        let path = Path::new(&self.config_path);

        let content = serde_yaml::to_string(&self.roles).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("序列化角色配置失败: {e}")))
        })?;

        fs::write(path, content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "写入角色配置文件失败: {e}"
            )))
        })?;

        Ok(())
    }

    /// 创建默认的角色配置YAML文件
    pub fn create_default_config(config_path: &str) -> AiResult<()> {
        let default_roles: HashMap<String, RoleConfig> = HashMap::new();
        let path = Path::new(config_path);

        let content = serde_yaml::to_string(&default_roles).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "序列化默认角色配置失败: {e}"
            )))
        })?;

        fs::write(path, content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "创建默认角色配置文件失败: {e}"
            )))
        })?;

        Ok(())
    }

    /// 验证YAML配置文件格式
    pub fn validate_config_file(config_path: &str) -> AiResult<()> {
        let path = Path::new(config_path);

        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                    "配置文件不存在: {config_path}",
                ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("读取配置文件失败: {e}")))
        })?;

        serde_yaml::from_str::<HashMap<String, RoleConfig>>(&content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("YAML格式验证失败: {e}")))
        })?;

        Ok(())
    }
    
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
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "角色配置文件不存在: {}",
                self.config_path
            ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "读取角色配置文件失败: {e}"
            )))
        })?;

        let roles_config: HashMap<String, RoleConfig> =
            serde_yaml::from_str(&content).map_err(|e| {
                AiError::from(AiErrReason::ConfigError(format!(
                    "解析角色配置文件失败: {e}"
                )))
            })?;

        self.roles = roles_config;
        Ok(())
    }

    /// 获取角色配置
    pub fn get_role_config(&self, role_key: &str) -> Option<&RoleConfig> {
        self.roles.get(role_key)
    }

    /// 加载规则配置文件
    pub fn load_rules_config(&self, rules_path: &str) -> AiResult<RulesConfig> {
        let path = Path::new(rules_path);

        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "规则配置路径不存在: {rules_path}"
            ))));
        }

        // 判断是文件还是目录
        if path.is_file() {
            // 如果是文件，直接读取内容到rules数组
            let content = fs::read_to_string(path).map_err(|e| {
                AiError::from(AiErrReason::ConfigError(format!(
                    "读取规则配置文件失败: {e}"
                )))
            })?;
            
            info!("加载角色RULE文件: {}", rules_path);
            // 将文件内容按行分割，过滤空行
            let rules: Vec<String> = content
                .lines()
                .map(|line| line.trim().to_string())
                .filter(|line| !line.is_empty())
                .collect();
            
            Ok(RulesConfig { rules })
        } else if path.is_dir() {
            // 如果是目录，加载所有*.mdc文件
            let mut rules = Vec::new();
            
            let entries = fs::read_dir(path).map_err(|e| {
                AiError::from(AiErrReason::ConfigError(format!(
                    "读取规则配置目录失败: {e}"
                )))
            })?;
            
            for entry in entries {
                let entry = entry.map_err(|e| {
                    AiError::from(AiErrReason::ConfigError(format!(
                        "读取目录条目失败: {e}"
                    )))
                })?;
                
                let file_path = entry.path();
                info!("加载角色RULE文件: {}", file_path.display());
                if file_path.extension().and_then(|s| s.to_str()) == Some("mdc") {
                    let content = fs::read_to_string(&file_path).map_err(|e| {
                        AiError::from(AiErrReason::ConfigError(format!(
                            "读取规则文件 {:?} 失败: {e}",
                            file_path.file_name().unwrap_or_default()
                        )))
                    })?;
                    
                    // 将文件内容按行分割，过滤空行
                    let file_rules: Vec<String> = content
                        .lines()
                        .map(|line| line.trim().to_string())
                        .filter(|line| !line.is_empty())
                        .collect();
                    
                    rules.extend(file_rules);
                }
            }
            
            Ok(RulesConfig { rules })
        } else {
            Err(AiError::from(AiErrReason::ConfigError(format!(
                "规则配置路径既不是文件也不是目录: {rules_path}"
            ))))
        }
    }

    /// 获取角色规则配置
    pub fn get_role_rules_config(&self, role_key: &str) -> AiResult<Option<RulesConfig>> {
        if let Some(role_config) = self.roles.get(role_key) {
            if let Some(rules_path) = &role_config.rules_config {
                let full_path = Path::new(&self.config_path)
                    .parent()
                    .unwrap()
                    .join(rules_path);
                
                info!("加载角色RULE: {role_key}" );
                let rules_config = self.load_rules_config(full_path.to_str().unwrap())?;
                Ok(Some(rules_config))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// 加载全局规则配置
    pub fn load_global_rules_config(&self) -> AiResult<RulesConfig> {
        let global_path = Path::new(&self.config_path)
            .parent()
            .unwrap()
            .join("rules/global.yaml");
        self.load_rules_config(global_path.to_str().unwrap())
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

    /// 自动检测并加载配置（只使用roles.yaml）
    pub fn auto_load_config(&mut self, _simplified_path: &str, legacy_path: &str) -> AiResult<()> {
        // 只加载传统配置
        if Path::new(legacy_path).exists() {
            println!("📄 Loading roles configuration from {legacy_path}...");
            return self.load_config();
        }
        
        Err(AiError::from(AiErrReason::ConfigError(
            format!("Configuration file not found: {legacy_path}")
        )))
    }
}

impl Default for RoleConfigManager {
    fn default() -> Self {
        Self::new("src/ai/config/roles.yaml".to_string())
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

    /// 自动加载角色配置管理器（只使用roles.yaml）
    pub fn auto_load(_simplified_path: Option<String>, legacy_path: Option<String>) -> AiResult<RoleConfigManager> {
        let legacy = legacy_path.unwrap_or_else(|| "src/ai/config/roles.yaml".to_string());
        let mut manager = RoleConfigManager::new(legacy.clone());
        println!("📄 Loading roles configuration from {legacy}...");
        manager.load_config()?;
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_role_config_manager() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

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
        assert_eq!(
            config.recommended_models,
            vec!["test-model", "backup-model"]
        );
    }

    #[test]
    fn test_role_config_manager_yaml() {
        let test_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "dev-model"
  recommended_models:
    - "dev-model"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.role_exists("developer"));
    }

    #[test]
    fn test_save_config() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path.clone());
        assert!(manager.load_config().is_ok());

        // Test saving config
        assert!(manager.save_config().is_ok());

        // Reload and verify
        let mut new_manager = RoleConfigManager::new(temp_path);
        assert!(new_manager.load_config().is_ok());
        assert!(new_manager.role_exists("test_role"));
    }

    #[test]
    fn test_role_config_loader_with_yaml() {
        let test_config = r#"
    analyst:
      name: "分析师"
      description: "专注于数据分析和洞察"
      system_prompt: "你是一个数据分析师，擅长从数据中发现规律和趋势"
      recommended_model: "gpt-4"
      recommended_models:
        - "gpt-4"
        - "claude-3-sonnet"
        - "deepseek-chat"
            "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.role_exists("analyst"));

        let role_config = manager.get_role_config("analyst");
        assert!(role_config.is_some());
        let config = role_config.unwrap();
        assert_eq!(config.name, "分析师");
        assert_eq!(config.recommended_model, "gpt-4");
        assert_eq!(
            config.recommended_models,
            vec!["gpt-4", "claude-3-sonnet", "deepseek-chat"]
        );
    }

    #[test]
    fn test_yaml_complex_roles() {
        let complex_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发和工程化"
  system_prompt: "你是一个资深的开发者，擅长编写高质量、可维护的代码"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3-sonnet"
    - "deepseek-coder"

writer:
  name: "作家"
  description: "专注于创意写作和文档生成"
  system_prompt: "你是一个专业的作家，擅长创作各种类型的文本内容"
  recommended_model: "claude-3-opus"
  recommended_models:
    - "claude-3-opus"
    - "gpt-4"
    - "deepseek-chat"

debugger:
  name: "调试专家"
  description: "专注于代码调试和问题排查"
  system_prompt: "你是一个资深的调试专家，擅长发现和解决代码中的各种问题"
  recommended_model: "deepseek-coder"
  recommended_models:
    - "deepseek-coder"
    - "gpt-4"
    - "claude-3-sonnet"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(complex_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path)).unwrap();

        // Test multiple roles
        assert!(manager.role_exists("developer"));
        assert!(manager.role_exists("writer"));
        assert!(manager.role_exists("debugger"));

        let dev_config = manager.get_role_config("developer").unwrap();
        assert_eq!(dev_config.name, "开发者");
        assert_eq!(dev_config.recommended_model, "gpt-4");
        assert_eq!(dev_config.recommended_models.len(), 3);

        let writer_config = manager.get_role_config("writer").unwrap();
        assert_eq!(writer_config.name, "作家");
        assert_eq!(writer_config.recommended_model, "claude-3-opus");

        let debugger_config = manager.get_role_config("debugger").unwrap();
        assert_eq!(debugger_config.name, "调试专家");
        assert_eq!(debugger_config.recommended_model, "deepseek-coder");
    }

    #[test]
    fn test_yaml_with_comments() {
        let config_with_comments = r#"# 这是角色配置文件的示例
# 包含多个AI角色定义

developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3"

# 这是分析师角色
analyst:
  name: "分析师"
  description: "数据分析"
  system_prompt: "你是一个分析师"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(config_with_comments.as_bytes())
            .unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path)).unwrap();

        assert!(manager.role_exists("developer"));
        assert!(manager.role_exists("analyst"));

        let dev_config = manager.get_role_config("developer").unwrap();
        assert_eq!(dev_config.name, "开发者");
        assert_eq!(dev_config.recommended_models, vec!["gpt-4", "claude-3"]);
    }

    #[test]
    fn test_create_and_validate_default_config() {
        let temp_path = "/tmp/test_roles_default.yaml";

        // Test creating default config
        let result = RoleConfigManager::create_default_config(temp_path);
        assert!(result.is_ok());

        // Test validating the created config
        let validate_result = RoleConfigManager::validate_config_file(temp_path);
        assert!(validate_result.is_ok());

        // Test loading the created config
        let mut manager = RoleConfigManager::new(temp_path.to_string());
        assert!(manager.load_config().is_ok());

        // Should have no roles in default config
        let available_roles = manager.get_available_roles();
        assert_eq!(available_roles.len(), 0);

        // Clean up
        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_invalid_yaml_format() {
        let invalid_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3"
  invalid_field: "this should cause an error"

analyst:
  name: "分析师"
  description: "数据分析"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path);
        let result = manager.load_config();

        // Should fail to parse due to invalid field
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_reload_cycle() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

        let _temp_path = "/tmp/test_roles_save_reload.yaml";

        // Create initial manager
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let original_path = temp_file.path().to_str().unwrap().to_string();

        // Load original config
        let mut manager = RoleConfigManager::new(original_path.clone());
        assert!(manager.load_config().is_ok());

        // Modify the config
        let available_roles = manager.get_available_roles();
        assert_eq!(available_roles.len(), 1);

        // Save config
        assert!(manager.save_config().is_ok());

        // Create new manager and reload
        let mut new_manager = RoleConfigManager::new(original_path.clone());
        assert!(new_manager.load_config().is_ok());

        // Verify data integrity
        assert!(new_manager.role_exists("test_role"));
        let reloaded_config = new_manager.get_role_config("test_role").unwrap();
        assert_eq!(reloaded_config.name, "测试角色");
        assert_eq!(
            reloaded_config.recommended_models,
            vec!["test-model", "backup-model"]
        );

        // Clean up
        std::fs::remove_file(original_path.clone()).unwrap();
    }
}
