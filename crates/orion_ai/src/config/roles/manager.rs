use crate::config::roles::types::{RoleConfig, RulesConfig};
use crate::error::{AiErrReason, AiError, AiResult};
use crate::AiRoleID;
use getset::Getters;
use log::info;
use orion_error::{ToStructError, UvsConfFrom};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// 角色配置管理器
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct RoleConfigManager {
    /// 角色配置映射
    default_role: AiRoleID,
    default_model: String,
    roles: HashMap<String, RoleConfig>,
}
impl Default for RoleConfigManager {
    fn default() -> Self {
        let model = "deepseek-chat".to_string();
        let roles = RoleConfig::example_roles();
        Self {
            default_role: AiRoleID::new("galactiward"),
            default_model: model,
            roles,
        }
    }
}

impl RoleConfigManager {
    /// 获取角色配置
    pub fn get_role_config(&self, role_key: &str) -> Option<&RoleConfig> {
        self.roles.get(role_key)
    }

    /// 加载规则配置文件
    pub fn load_rules_config(&self, rules_path: &PathBuf) -> AiResult<RulesConfig> {
        let path = Path::new(rules_path);

        if !path.exists() {
            return Err(AiErrReason::from_conf(format!(
                "规则配置路径不存在: {}",
                rules_path.display()
            ))
            .to_err());
        }

        // 判断是文件还是目录
        if path.is_file() {
            // 如果是文件，直接读取内容到rules数组
            let content = fs::read_to_string(path).map_err(|e| {
                AiError::from(AiErrReason::from_conf(format!("读取规则配置文件失败: {e}")))
            })?;

            info!("加载角色RULE文件: {}", rules_path.display());
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
                AiError::from(AiErrReason::from_conf(format!("读取规则配置目录失败: {e}")))
            })?;

            for entry in entries {
                let entry = entry.map_err(|e| {
                    AiError::from(AiErrReason::from_conf(format!("读取目录条目失败: {e}")))
                })?;

                let file_path = entry.path();
                info!("加载角色RULE文件: {}", file_path.display());
                if file_path.extension().and_then(|s| s.to_str()) == Some("mdc") {
                    let content = fs::read_to_string(&file_path).map_err(|e| {
                        AiError::from(AiErrReason::from_conf(format!(
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
            Err(AiError::from(AiErrReason::from_conf(format!(
                "规则配置路径既不是文件也不是目录: {}",
                rules_path.display()
            ))))
        }
    }

    /// 获取角色规则配置
    pub fn get_role_rules_config(&self, role_key: &str) -> AiResult<Option<RulesConfig>> {
        if let Some(role_config) = self.roles.get(role_key) {
            if let Some(rules_path) = role_config.rules_path() {
                // 使用分层规则配置路径
                let layered_rules_path =
                    crate::config::roles::loader::RoleConfigLoader::get_layered_rules_path(
                        rules_path,
                    )?;

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

    /// 获取所有可用的角色
    pub fn get_available_roles(&self) -> Vec<&String> {
        self.roles.keys().collect()
    }

    /// 检查角色是否存在
    pub fn role_exists(&self, role_key: &str) -> bool {
        self.roles.contains_key(role_key)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use orion_common::serde::Yamlable;
    use orion_error::TestAssert;

    use super::RoleConfigManager;

    #[test]
    fn example_save_load() {
        let path = PathBuf::from("./examples/ai-roles.yml");
        RoleConfigManager::default().save_yml(&path).assert();
        let roles_mng = RoleConfigManager::from_yml(&path).assert();
        println!("roles: {roles_mng:#?}");
    }
}
