use serde::{Deserialize, Serialize};

/// AI任务枚举 - 定义各种AI任务类型和用途
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiTask {
    // 代码开发类
    Coding,      // 代码生成和实现
    Reviewing,   // 代码审查和评估
    Testing,     // 测试用例生成和执行
    Documenting, // 文档生成和维护
    
    // 项目管理类
    Committing,  // Git提交信息生成
    Branching,   // 分支管理建议
    Releasing,   // 发布规划和执行
    Planning,    // 项目规划和任务分解
    
    // 系统运维类
    Installing,  // 软件安装和配置
    Deploying,   // 应用部署和发布
    Configuring, // 系统配置优化
    Restarting,  // 服务重启和管理
    
    // 监控诊断类
    Monitoring,  // 系统监控和告警
    Troubleshooting, // 故障排查和修复
    Analyzing,   // 深度分析和诊断
    Auditing,    // 安全审计和合规检查
    
    // 知识管理类
    Explaining,  // 知识解释和教学
    Searching,   // 信息检索和知识发现
    Optimizing,  // 性能优化和改进
    Learning,    // 学习和知识获取
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

impl AiTask {
    /// 检查该任务是否需要完整代码上下文
    pub fn needs_full_context(&self) -> bool {
        matches!(
            self,
            AiTask::Reviewing | AiTask::Analyzing | AiTask::Planning
        )
    }

    /// 检查该任务是否对token敏感
    pub fn is_token_sensitive(&self) -> bool {
        matches!(
            self,
            AiTask::Coding | AiTask::Documenting | AiTask::Explaining
        )
    }

    /// 获取该任务的推荐模型
    pub fn recommended_model(&self) -> &'static str {
        match self {
            AiTask::Coding => "gpt-4o",
            AiTask::Reviewing => "claude-3-5-sonnet",
            AiTask::Testing => "gpt-4o",
            AiTask::Documenting => "gpt-4o-mini",
            AiTask::Committing => "gpt-4o-mini",
            AiTask::Branching => "gpt-4o-mini",
            AiTask::Releasing => "gpt-4o",
            AiTask::Planning => "gpt-4o",
            AiTask::Installing => "gpt-4o-mini",
            AiTask::Deploying => "gpt-4o-mini",
            AiTask::Configuring => "gpt-4o",
            AiTask::Restarting => "gpt-4o-mini",
            AiTask::Monitoring => "gpt-4o",
            AiTask::Troubleshooting => "claude-3-5-sonnet",
            AiTask::Analyzing => "claude-3-5-sonnet",
            AiTask::Auditing => "gpt-4o",
            AiTask::Explaining => "gpt-4o-mini",
            AiTask::Searching => "gpt-4o-mini",
            AiTask::Optimizing => "claude-3-5-sonnet",
            AiTask::Learning => "gpt-4o-mini",
        }
    }

    /// 获取任务的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            AiTask::Coding => "coding",
            AiTask::Reviewing => "reviewing",
            AiTask::Testing => "testing",
            AiTask::Documenting => "documenting",
            AiTask::Committing => "committing",
            AiTask::Branching => "branching",
            AiTask::Releasing => "releasing",
            AiTask::Planning => "planning",
            AiTask::Installing => "installing",
            AiTask::Deploying => "deploying",
            AiTask::Configuring => "configuring",
            AiTask::Restarting => "restarting",
            AiTask::Monitoring => "monitoring",
            AiTask::Troubleshooting => "troubleshooting",
            AiTask::Analyzing => "analyzing",
            AiTask::Auditing => "auditing",
            AiTask::Explaining => "explaining",
            AiTask::Searching => "searching",
            AiTask::Optimizing => "optimizing",
            AiTask::Learning => "learning",
        }
    }

    /// 获取优化的推荐模型列表（中国大陆优先）
    pub fn recommended_models(&self) -> Vec<&'static str> {
        match self {
            AiTask::Coding => vec!["deepseek-coder", "gpt-4o", "claude-3-5-sonnet"],
            AiTask::Reviewing => vec!["deepseek-chat", "claude-3-5-sonnet", "gpt-4o"],
            AiTask::Testing => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Documenting => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Committing => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Branching => vec!["gpt-4o-mini", "deepseek-chat"],
            AiTask::Releasing => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Planning => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Installing => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Deploying => vec!["gpt-4o-mini", "deepseek-chat", "qwen-max"],
            AiTask::Configuring => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Restarting => vec!["gpt-4o-mini", "deepseek-chat"],
            AiTask::Monitoring => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Troubleshooting => vec!["claude-3-5-sonnet", "deepseek-chat", "gpt-4o"],
            AiTask::Analyzing => vec!["claude-3-5-sonnet", "deepseek-chat", "gpt-4o"],
            AiTask::Auditing => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Explaining => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Searching => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Optimizing => vec!["claude-3-5-sonnet", "deepseek-coder", "gpt-4o"],
            AiTask::Learning => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
        }
    }

    /// 检查模型是否在中国可用
    pub fn is_available_in_china(&self, model: &str) -> bool {
        matches!(model, "deepseek-chat" | "deepseek-coder" | "qwen-turbo" | "qwen-max")
    }

    /// 检查模型是否对中国用户友好
    pub fn is_china_friendly(&self, model: &str) -> bool {
        self.is_available_in_china(model)
    }

    /// 获取中国用户的注意事项
    pub fn get_china_notes(&self) -> &'static str {
        "建议使用国内模型以获得更好的访问速度和稳定性"
    }

    /// 从旧能力转换为新任务
    pub fn from_legacy(capability: LegacyAiDevCapability) -> Self {
        match capability {
            LegacyAiDevCapability::Analyze => AiTask::Analyzing,
            LegacyAiDevCapability::Suggest => AiTask::Optimizing,
            LegacyAiDevCapability::Check => AiTask::Reviewing,
            LegacyAiDevCapability::Generate => AiTask::Coding,
            LegacyAiDevCapability::Refactor => AiTask::Coding,
            LegacyAiDevCapability::Deploy => AiTask::Deploying,
            LegacyAiDevCapability::Commit => AiTask::Committing,
            LegacyAiDevCapability::Review => AiTask::Reviewing,
            LegacyAiDevCapability::Understand => AiTask::Explaining,
            LegacyAiDevCapability::Predict => AiTask::Planning,
            LegacyAiDevCapability::Collaborate => AiTask::Learning,
            LegacyAiDevCapability::Explain => AiTask::Explaining,
        }
    }
}

impl std::fmt::Display for AiTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AiTask {
    type Err = String;

    /// 从字符串解析 AiTask
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "coding" => Ok(AiTask::Coding),
            "reviewing" => Ok(AiTask::Reviewing),
            "testing" => Ok(AiTask::Testing),
            "documenting" => Ok(AiTask::Documenting),
            "committing" => Ok(AiTask::Committing),
            "branching" => Ok(AiTask::Branching),
            "releasing" => Ok(AiTask::Releasing),
            "planning" => Ok(AiTask::Planning),
            "installing" => Ok(AiTask::Installing),
            "deploying" => Ok(AiTask::Deploying),
            "configuring" => Ok(AiTask::Configuring),
            "restarting" => Ok(AiTask::Restarting),
            "monitoring" => Ok(AiTask::Monitoring),
            "troubleshooting" => Ok(AiTask::Troubleshooting),
            "analyzing" => Ok(AiTask::Analyzing),
            "auditing" => Ok(AiTask::Auditing),
            "explaining" => Ok(AiTask::Explaining),
            "searching" => Ok(AiTask::Searching),
            "optimizing" => Ok(AiTask::Optimizing),
            "learning" => Ok(AiTask::Learning),
            // 向后兼容：支持旧的能力名称
            "analyze" => Ok(AiTask::Analyzing),
            "suggest" => Ok(AiTask::Optimizing),
            "check" => Ok(AiTask::Reviewing),
            "generate" => Ok(AiTask::Coding),
            "refactor" => Ok(AiTask::Coding),
            "deploy" => Ok(AiTask::Deploying),
            "commit" => Ok(AiTask::Committing),
            "review" => Ok(AiTask::Reviewing),
            "understand" => Ok(AiTask::Explaining),
            "predict" => Ok(AiTask::Planning),
            "collaborate" => Ok(AiTask::Learning),
            "explain" => Ok(AiTask::Explaining),
            _ => Err(format!("Unknown task: {s}")),
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
