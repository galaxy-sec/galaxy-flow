use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// AI角色ID结构体 - 完全动态的角色标识系统
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct AiRoleID {
    /// 角色唯一标识符
    id: String,
}

impl AiRoleID {
    /// 创建新的角色ID
    pub fn new<S: Into<String>>(id: S) -> Self {
        Self { id: id.into() }
    }
    
    /// 获取角色ID
    pub fn id(&self) -> &str {
        &self.id
    }
    
    /// 获取角色的描述信息
    pub fn description(&self) -> String {
        format!("角色: {}", self.id)
    }
    
    /// 获取角色的字符串表示
    pub fn as_str(&self) -> &str {
        &self.id
    }
    
    /// 检查是否为预定义角色
    pub fn is_predefined(&self) -> bool {
        matches!(self.id.as_str(), "developer" | "operations" | "knowledger" | "galactiward")
    }
}

impl std::fmt::Display for AiRoleID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl FromStr for AiRoleID {
    type Err = String;

    /// 从字符串解析 AiRoleID
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            return Err("角色ID不能为空".to_string());
        }
        Ok(Self::new(s.trim().to_string()))
    }
}

impl<S: Into<String>> From<S> for AiRoleID {
    fn from(id: S) -> Self {
        Self::new(id)
    }
}

/// 向后兼容的类型别名
pub type AiRole = AiRoleID;
