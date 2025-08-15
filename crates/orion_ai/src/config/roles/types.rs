use getset::Getters;
use serde::{Deserialize, Serialize};
/// 角色配置结构
#[derive(Debug, Clone, Serialize, Deserialize, Getters)]
#[getset(get = "pub")]
pub struct RoleConfig {
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 推荐模型
    pub used_model: String,
    /// 规则配置文件路径
    pub rules_path: Option<String>,
}

/// 规则配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RulesConfig {
    /// 规则集合
    pub rules: Vec<String>,
}
