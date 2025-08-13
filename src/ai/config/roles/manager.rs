use crate::ai::error::{AiErrReason, AiError, AiResult};
use crate::ai::config::roles::types::{RoleConfig, RulesConfig};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

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
                    AiError::from(AiErrReason::ConfigError(format!("读取目录条目失败: {e}")))
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
            if let Some(rules_path) = &role_config.rules_path {
                // 使用分层规则配置路径
                let layered_rules_path = crate::ai::config::roles::loader::RoleConfigLoader::get_layered_rules_path(rules_path)?;

                info!("加载角色RULE: {role_key}");
                let rules_config = self.load_rules_config(&layered_rules_path)?;
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

        Err(AiError::from(AiErrReason::ConfigError(format!(
            "Configuration file not found: {legacy_path}"
        ))))
    }
}

impl Default for RoleConfigManager {
    fn default() -> Self {
        Self::new("src/ai/config/roles.yaml".to_string())
    }
}