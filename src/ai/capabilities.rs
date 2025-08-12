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

/// AI任务枚举 - 定义各种AI任务类型和用途
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiTask {
    // 开发者场景任务
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
    
    // 代码优化类
    Optimizing,  // 性能优化和改进
    Refactoring, // 代码重构和改进
    Debugging,   // 代码调试和问题修复
    
    // 运维人员场景任务
    // 系统部署类
    Installing,  // 软件安装和配置
    Deploying,   // 应用部署和发布
    Configuring, // 系统配置优化
    Restarting,  // 服务重启和管理
    
    // 监控诊断类
    Monitoring,  // 系统监控和告警
    Troubleshooting, // 故障排查和修复
    Analyzing,   // 深度分析和诊断
    Auditing,    // 安全审计和合规检查
    
    // 运维管理类
    BackingUp,   // 系统备份和恢复
    Scaling,     // 系统扩容和缩容
    Securing,    // 系统安全加固
    
    // 通用知识管理任务
    Explaining,  // 知识解释和教学
    Searching,   // 信息检索和知识发现
    Learning,    // 学习和知识获取
    Consulting,  // 专业咨询和建议
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

    /// 获取该角色支持的所有任务类型
    pub fn supported_tasks(&self) -> Vec<AiTask> {
        match self {
            AiRole::Developer => vec![
                // 代码开发类
                AiTask::Coding,
                AiTask::Reviewing,
                AiTask::Testing,
                AiTask::Documenting,
                // 项目管理类
                AiTask::Committing,
                AiTask::Branching,
                AiTask::Releasing,
                AiTask::Planning,
                // 代码优化类
                AiTask::Optimizing,
                AiTask::Refactoring,
                AiTask::Debugging,
            ],
            AiRole::Operations => vec![
                // 系统部署类
                AiTask::Installing,
                AiTask::Deploying,
                AiTask::Configuring,
                AiTask::Restarting,
                // 监控诊断类
                AiTask::Monitoring,
                AiTask::Troubleshooting,
                AiTask::Analyzing,
                AiTask::Auditing,
                // 运维管理类
                AiTask::BackingUp,
                AiTask::Scaling,
                AiTask::Securing,
            ],
            AiRole::KnowledgeManager => vec![
                // 通用知识管理任务
                AiTask::Explaining,
                AiTask::Searching,
                AiTask::Learning,
                AiTask::Consulting,
            ],
        }
    }

    /// 根据用户输入智能选择合适的任务类型
    pub fn infer_task(&self, user_input: &str) -> AiTask {
        let input_lower = user_input.to_lowercase();
        
        match self {
            AiRole::Developer => {
                // 开发者场景的任务推断逻辑
                if input_lower.contains("代码") || input_lower.contains("实现") || input_lower.contains("编写") {
                    AiTask::Coding
                } else if input_lower.contains("审查") || input_lower.contains("review") || input_lower.contains("检查") {
                    AiTask::Reviewing
                } else if input_lower.contains("测试") || input_lower.contains("test") || input_lower.contains("用例") {
                    AiTask::Testing
                } else if input_lower.contains("文档") || input_lower.contains("document") || input_lower.contains("说明") {
                    AiTask::Documenting
                } else if input_lower.contains("提交") || input_lower.contains("commit") || input_lower.contains("git") {
                    AiTask::Committing
                } else if input_lower.contains("分支") || input_lower.contains("branch") || input_lower.contains("合并") {
                    AiTask::Branching
                } else if input_lower.contains("发布") || input_lower.contains("release") || input_lower.contains("版本") {
                    AiTask::Releasing
                } else if input_lower.contains("规划") || input_lower.contains("plan") || input_lower.contains("设计") {
                    AiTask::Planning
                } else if input_lower.contains("优化") || input_lower.contains("optimize") || input_lower.contains("性能") {
                    AiTask::Optimizing
                } else if input_lower.contains("重构") || input_lower.contains("refactor") || input_lower.contains("结构") {
                    AiTask::Refactoring
                } else if input_lower.contains("调试") || input_lower.contains("debug") || input_lower.contains("问题") {
                    AiTask::Debugging
                } else {
                    // 默认选择最常用的开发者任务
                    AiTask::Coding
                }
            },
            AiRole::Operations => {
                // 运维人员场景的任务推断逻辑
                if input_lower.contains("安装") || input_lower.contains("install") || input_lower.contains("环境") {
                    AiTask::Installing
                } else if input_lower.contains("部署") || input_lower.contains("deploy") || input_lower.contains("发布") {
                    AiTask::Deploying
                } else if input_lower.contains("配置") || input_lower.contains("config") || input_lower.contains("设置") {
                    AiTask::Configuring
                } else if input_lower.contains("重启") || input_lower.contains("restart") || input_lower.contains("服务") {
                    AiTask::Restarting
                } else if input_lower.contains("监控") || input_lower.contains("monitor") || input_lower.contains("告警") {
                    AiTask::Monitoring
                } else if input_lower.contains("故障") || input_lower.contains("trouble") || input_lower.contains("排查") {
                    AiTask::Troubleshooting
                } else if input_lower.contains("分析") || input_lower.contains("analyze") || input_lower.contains("诊断") {
                    AiTask::Analyzing
                } else if input_lower.contains("审计") || input_lower.contains("audit") || input_lower.contains("安全") {
                    AiTask::Auditing
                } else if input_lower.contains("备份") || input_lower.contains("backup") || input_lower.contains("恢复") {
                    AiTask::BackingUp
                } else if input_lower.contains("扩容") || input_lower.contains("scale") || input_lower.contains("性能") {
                    AiTask::Scaling
                } else if input_lower.contains("加固") || input_lower.contains("secure") || input_lower.contains("防护") {
                    AiTask::Securing
                } else {
                    // 默认选择最常用的运维任务
                    AiTask::Deploying
                }
            },
            AiRole::KnowledgeManager => {
                // 知识管理场景的任务推断逻辑
                if input_lower.contains("解释") || input_lower.contains("explain") || input_lower.contains("说明") {
                    AiTask::Explaining
                } else if input_lower.contains("搜索") || input_lower.contains("search") || input_lower.contains("查找") {
                    AiTask::Searching
                } else if input_lower.contains("学习") || input_lower.contains("learn") || input_lower.contains("教程") {
                    AiTask::Learning
                } else if input_lower.contains("咨询") || input_lower.contains("consult") || input_lower.contains("建议") {
                    AiTask::Consulting
                } else {
                    // 默认选择最常用的知识管理任务
                    AiTask::Explaining
                }
            },
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

impl AiTask {
    /// 检查该任务是否需要完整代码上下文
    pub fn needs_full_context(&self) -> bool {
        matches!(
            self,
            // 需要完整上下文的分析和审查类任务
            AiTask::Reviewing | AiTask::Analyzing | AiTask::Planning |
            AiTask::Refactoring | AiTask::Debugging | AiTask::Troubleshooting |
            AiTask::Auditing | AiTask::Optimizing | AiTask::Consulting
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
            // 开发者场景任务
            AiTask::Coding => "deepseek-coder",
            AiTask::Reviewing => "claude-3-5-sonnet",
            AiTask::Testing => "gpt-4o",
            AiTask::Documenting => "gpt-4o-mini",
            AiTask::Committing => "gpt-4o-mini",
            AiTask::Branching => "gpt-4o-mini",
            AiTask::Releasing => "gpt-4o",
            AiTask::Planning => "gpt-4o",
            AiTask::Optimizing => "claude-3-5-sonnet",
            AiTask::Refactoring => "deepseek-coder",
            AiTask::Debugging => "claude-3-5-sonnet",
            
            // 运维人员场景任务
            AiTask::Installing => "gpt-4o-mini",
            AiTask::Deploying => "gpt-4o-mini",
            AiTask::Configuring => "gpt-4o",
            AiTask::Restarting => "gpt-4o-mini",
            AiTask::Monitoring => "gpt-4o",
            AiTask::Troubleshooting => "claude-3-5-sonnet",
            AiTask::Analyzing => "claude-3-5-sonnet",
            AiTask::Auditing => "gpt-4o",
            AiTask::BackingUp => "gpt-4o-mini",
            AiTask::Scaling => "gpt-4o",
            AiTask::Securing => "gpt-4o",
            
            // 通用知识管理任务
            AiTask::Explaining => "gpt-4o-mini",
            AiTask::Searching => "gpt-4o-mini",
            AiTask::Learning => "gpt-4o-mini",
            AiTask::Consulting => "gpt-4o-mini",
        }
    }

    /// 获取任务的字符串表示
    pub fn as_str(&self) -> &'static str {
        match self {
            // 开发者场景任务
            AiTask::Coding => "coding",
            AiTask::Reviewing => "reviewing",
            AiTask::Testing => "testing",
            AiTask::Documenting => "documenting",
            AiTask::Committing => "committing",
            AiTask::Branching => "branching",
            AiTask::Releasing => "releasing",
            AiTask::Planning => "planning",
            AiTask::Optimizing => "optimizing",
            AiTask::Refactoring => "refactoring",
            AiTask::Debugging => "debugging",
            
            // 运维人员场景任务
            AiTask::Installing => "installing",
            AiTask::Deploying => "deploying",
            AiTask::Configuring => "configuring",
            AiTask::Restarting => "restarting",
            AiTask::Monitoring => "monitoring",
            AiTask::Troubleshooting => "troubleshooting",
            AiTask::Analyzing => "analyzing",
            AiTask::Auditing => "auditing",
            AiTask::BackingUp => "backing_up",
            AiTask::Scaling => "scaling",
            AiTask::Securing => "securing",
            
            // 通用知识管理任务
            AiTask::Explaining => "explaining",
            AiTask::Searching => "searching",
            AiTask::Learning => "learning",
            AiTask::Consulting => "consulting",
        }
    }

    /// 获取优化的推荐模型列表（中国大陆优先）
    pub fn recommended_models(&self) -> Vec<&'static str> {
        match self {
            // 开发者场景任务
            AiTask::Coding => vec!["deepseek-coder", "claude-3-5-sonnet", "gpt-4o"],
            AiTask::Reviewing => vec!["claude-3-5-sonnet", "deepseek-chat", "gpt-4o"],
            AiTask::Testing => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Documenting => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Committing => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Branching => vec!["gpt-4o-mini", "deepseek-chat"],
            AiTask::Releasing => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Planning => vec!["gpt-4o", "deepseek-chat", "claude-3-5-sonnet"],
            AiTask::Optimizing => vec!["claude-3-5-sonnet", "deepseek-coder", "gpt-4o"],
            AiTask::Refactoring => vec!["deepseek-coder", "claude-3-5-sonnet", "gpt-4o"],
            AiTask::Debugging => vec!["claude-3-5-sonnet", "deepseek-coder", "gpt-4o"],
            
            // 运维人员场景任务
            AiTask::Installing => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Deploying => vec!["gpt-4o-mini", "deepseek-chat", "qwen-max"],
            AiTask::Configuring => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Restarting => vec!["gpt-4o-mini", "deepseek-chat"],
            AiTask::Monitoring => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Troubleshooting => vec!["claude-3-5-sonnet", "deepseek-chat", "gpt-4o"],
            AiTask::Analyzing => vec!["claude-3-5-sonnet", "deepseek-chat", "gpt-4o"],
            AiTask::Auditing => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::BackingUp => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Scaling => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            AiTask::Securing => vec!["gpt-4o", "deepseek-chat", "qwen-max"],
            
            // 通用知识管理任务
            AiTask::Explaining => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Searching => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Learning => vec!["gpt-4o-mini", "deepseek-chat", "qwen-turbo"],
            AiTask::Consulting => vec!["gpt-4o-mini", "deepseek-chat", "glm-4"],
        }
    }

    /// 获取该任务所属的角色
    pub fn get_role(&self) -> AiRole {
        match self {
            // 开发者场景任务
            AiTask::Coding | AiTask::Reviewing | AiTask::Testing | AiTask::Documenting |
            AiTask::Committing | AiTask::Branching | AiTask::Releasing | AiTask::Planning |
            AiTask::Optimizing | AiTask::Refactoring | AiTask::Debugging => AiRole::Developer,
            
            // 运维人员场景任务
            AiTask::Installing | AiTask::Deploying | AiTask::Configuring | AiTask::Restarting |
            AiTask::Monitoring | AiTask::Troubleshooting | AiTask::Analyzing | AiTask::Auditing |
            AiTask::BackingUp | AiTask::Scaling | AiTask::Securing => AiRole::Operations,
            
            // 通用知识管理任务
            AiTask::Explaining | AiTask::Searching | AiTask::Learning | AiTask::Consulting => AiRole::KnowledgeManager,
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
            LegacyAiDevCapability::Refactor => AiTask::Refactoring, // 更新：重构任务现在映射到Refactoring
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
