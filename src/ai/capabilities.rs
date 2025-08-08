use serde::{Deserialize, Serialize};

/// AI能力枚举 - 定义各种AI功能和用途
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiDevCapability {
    Analyze,     // 深度代码理解分析
    Suggest,     // 基于上下文的智能建议
    Check,       // 问题检测和审查
    Generate,    // 代码/文档创建
    Refactor,    // 重构建议
    Deploy,      // 智能部署决策
    Commit,      // Git提交信息生成
    Review,      // 代码审查
    Understand,  // 项目架构理解
    Predict,     // 变更影响分析
    Collaborate, // 团队协作增强
    Explain,
}

impl AiDevCapability {
    /// 检查该能力是否需要完整代码上下文
    pub fn needs_full_context(&self) -> bool {
        matches!(
            self,
            AiDevCapability::Review | AiDevCapability::Understand | AiDevCapability::Predict
        )
    }

    /// 检查该能力是否对token敏感
    pub fn is_token_sensitive(&self) -> bool {
        matches!(
            self,
            AiDevCapability::Generate | AiDevCapability::Refactor | AiDevCapability::Understand
        )
    }

    /// 获取该能力的推荐模型
    pub fn recommended_model(&self) -> &'static str {
        match self {
            AiDevCapability::Analyze => "gpt-4o-mini",
            AiDevCapability::Suggest => "gpt-4o-mini",
            AiDevCapability::Check => "gpt-4o",
            AiDevCapability::Generate => "gpt-4o",
            AiDevCapability::Refactor => "claude-3-5-sonnet",
            AiDevCapability::Deploy => "gpt-4o-mini",
            AiDevCapability::Commit => "gpt-4o-mini",
            AiDevCapability::Review => "claude-3-5-sonnet",
            AiDevCapability::Understand => "claude-3-5-sonnet",
            AiDevCapability::Predict => "gpt-4o",
            AiDevCapability::Collaborate => "gpt-4o-mini",
            AiDevCapability::Explain => "gpt-4o-mini",
        }
    }

    /// 获取能力的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            AiDevCapability::Analyze => "analyze",
            AiDevCapability::Suggest => "suggest",
            AiDevCapability::Check => "check",
            AiDevCapability::Generate => "generate",
            AiDevCapability::Refactor => "refactor",
            AiDevCapability::Deploy => "deploy",
            AiDevCapability::Commit => "commit",
            AiDevCapability::Review => "review",
            AiDevCapability::Understand => "understand",
            AiDevCapability::Predict => "predict",
            AiDevCapability::Collaborate => "collaborate",
            AiDevCapability::Explain => "explain",
        }
    }
}

impl std::fmt::Display for AiDevCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AiDevCapability {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "analyze" => Ok(AiDevCapability::Analyze),
            "suggest" => Ok(AiDevCapability::Suggest),
            "check" => Ok(AiDevCapability::Check),
            "generate" => Ok(AiDevCapability::Generate),
            "refactor" => Ok(AiDevCapability::Refactor),
            "deploy" => Ok(AiDevCapability::Deploy),
            "commit" => Ok(AiDevCapability::Commit),
            "review" => Ok(AiDevCapability::Review),
            "understand" => Ok(AiDevCapability::Understand),
            "predict" => Ok(AiDevCapability::Predict),
            "collaborate" => Ok(AiDevCapability::Collaborate),
            _ => Err(format!("Unknown capability: {}", s)),
        }
    }
}
