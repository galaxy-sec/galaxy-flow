use std::collections::HashMap;

use getset::Getters;
use serde::{Deserialize, Serialize};

/// 角色配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct RoleConfig {
    /// 角色名称
    name: String,
    /// 角色描述
    description: String,
    /// 系统提示词
    system_prompt: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    used_model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    rules_path: Option<String>,
}

/// 规则配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// 规则集合
    pub rules: Vec<String>,
}

impl RoleConfig {
    pub fn example_roles() -> HashMap<String, RoleConfig> {
        let mut roles = HashMap::new();
        roles.insert(
            "developer".to_string(),
            RoleConfig {
                name: "developer".to_string(),
                description: "专注于代码开发的技术专家".to_string(),
                system_prompt:
                    "你是一个专业的开发者，擅长高质量的代码实现、系统设计和技术问题解决。"
                        .to_string(),
                rules_path: Some("ai-rules/developer".to_string()),
                used_model: None,
            },
        );
        roles.insert(
            "operations".to_string(),
            RoleConfig {
                name: "operations".to_string(),
                description: "专注于系统运维的专家".to_string(),
                system_prompt: "你是一个专业的运维专家，擅长诊断系统问题、和解决问题。".to_string(),
                rules_path: Some("ai-rules/operations".to_string()),
                used_model: None,
            },
        );

        roles.insert(
            "galactiward".to_string(),
            RoleConfig {
                name: "galactiward".to_string(),
                description: "专注于Galaxy生态专家".to_string(),
                system_prompt: "通过Galaxy资料，解决Galaxy问题".to_string(),
                rules_path: Some("ai-rules/galactiward".to_string()),
                used_model: None,
            },
        );
        roles
    }
}
