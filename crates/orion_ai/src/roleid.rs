use serde::{Deserialize, Serialize};

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
        format!("ai-role: {}", self.id)
    }

    /// 获取角色的字符串表示
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl std::fmt::Display for AiRoleID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)
    }
}

/// 向后兼容的类型别名
pub type AiRole = AiRoleID;
