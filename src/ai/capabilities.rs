use serde::{Deserialize, Serialize};

/// AI角色枚举 - 定义用户可以选择的角色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiRole {
    /// 开发者角色 - 专注于代码开发、优化和项目管理
    Developer,
    /// 运维人员角色 - 专注于系统部署、监控和维护
    Operations,
    /// 通用知识管理角色 - 专注于知识获取、解释和咨询
    KnowledgeManager,
}

/// 旧版本的能力枚举，用于向后兼容
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum LegacyAiDevCapability {
    Analyze,
    Suggest,
    Check,
    Generate,
    Refactor,
    Deploy,
    Commit,
    Review,
    Understand,
    Predict,
    Collaborate,
    Explain,
}

impl AiRole {
    /// 获取角色的描述信息
    pub fn description(&self) -> &'static str {
        match self {
            AiRole::Developer => "开发者角色 - 专注于代码开发、优化和项目管理",
            AiRole::Operations => "运维人员角色 - 专注于系统部署、监控和维护",
            AiRole::KnowledgeManager => "通用知识管理角色 - 专注于知识获取、解释和咨询",
        }
    }

    /// 获取角色的推荐模型
    pub fn recommended_model(&self) -> &'static str {
        match self {
            AiRole::Developer => "deepseek-coder", // 开发者首选代码专用模型
            AiRole::Operations => "gpt-4o",        // 运维需要综合能力强的模型
            AiRole::KnowledgeManager => "gpt-4o-mini", // 知识管理使用轻量级模型
        }
    }

    /// 获取角色的推荐模型列表（中国大陆优先）
    pub fn recommended_models(&self) -> Vec<&'static str> {
        match self {
            AiRole::Developer => vec!["deepseek-coder", "claude-3-5-sonnet", "gpt-4o"],
            AiRole::Operations => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiRole::KnowledgeManager => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
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
            "knowledge_manager" | "km" | "知识管理" | "知识管理员" => Ok(AiRole::KnowledgeManager),
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
            AiRole::KnowledgeManager => "knowledge_manager",
        }
    }
}



// 为向后兼容性实现 FromStr
impl std::str::FromStr for LegacyAiDevCapability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "analyze" => Ok(LegacyAiDevCapability::Analyze),
            "suggest" => Ok(LegacyAiDevCapability::Suggest),
            "check" => Ok(LegacyAiDevCapability::Check),
            "generate" => Ok(LegacyAiDevCapability::Generate),
            "refactor" => Ok(LegacyAiDevCapability::Refactor),
            "deploy" => Ok(LegacyAiDevCapability::Deploy),
            "commit" => Ok(LegacyAiDevCapability::Commit),
            "review" => Ok(LegacyAiDevCapability::Review),
            "understand" => Ok(LegacyAiDevCapability::Understand),
            "predict" => Ok(LegacyAiDevCapability::Predict),
            "collaborate" => Ok(LegacyAiDevCapability::Collaborate),
            "explain" => Ok(LegacyAiDevCapability::Explain),
            _ => Err(format!("Unknown capability: {s}")),
        }
    }
}
