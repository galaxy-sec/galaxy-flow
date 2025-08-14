use serde::{Deserialize, Serialize};

/// AI角色枚举 - 定义用户可以选择的角色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiRole {
    /// 开发者角色 - 专注于代码开发、优化和项目管理
    Developer,
    /// 运维人员角色 - 专注于系统部署、监控和维护
    Operations,
    /// 通用知识管理角色 - 专注于知识获取、解释和咨询
    Knowledger,
    /// Galaxy Operator Ecosystem 专家
    GalactiWard,
}

impl AiRole {
    /// 获取角色的描述信息
    pub fn description(&self) -> &'static str {
        match self {
            AiRole::Developer => "开发者角色 - 专注于代码开发、优化和项目管理",
            AiRole::Operations => "运维人员角色 - 专注于系统部署、监控和维护",
            AiRole::Knowledger => "通用知识管理角色 - 专注于知识获取、解释和咨询",
            AiRole::GalactiWard => "Galaxy专家 - 专注于 Galaxy 生态的使用的诊断和建议",
        }
    }

    /// 获取角色的推荐模型
    pub fn recommended_model(&self) -> &'static str {
        match self {
            AiRole::Developer => "deepseek-coder", // 开发者首选代码专用模型
            AiRole::Operations => "gpt-4o",        // 运维需要综合能力强的模型
            AiRole::Knowledger => "gpt-4o-mini",   // 知识管理使用轻量级模型
            AiRole::GalactiWard => "gml-4.5",
        }
    }
}

impl std::fmt::Display for AiRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AiRole {
    type Err = String;

    /// 从字符串解析 AiRole
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "developer" | "dev" | "开发" | "开发者" => Ok(AiRole::Developer),
            "operations" | "ops" | "运维" | "运维人员" => Ok(AiRole::Operations),
            "knowledger" | "km" | "知识管理" | "知识管理员" => Ok(AiRole::Knowledger),
            "galactiward" | "galaxy" | "gxl" | "gflow" | "gops" => Ok(AiRole::GalactiWard),
            _ => Err(format!("Unknown role: {s}")),
        }
    }
}

impl AiRole {
    /// 获取角色的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            AiRole::Developer => "developer",
            AiRole::Operations => "operations",
            AiRole::Knowledger => "knowledger",
            AiRole::GalactiWard => "galaxyor",
        }
    }
}
