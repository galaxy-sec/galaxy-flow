use crate::ai::error::{AiErrReason, AiError, AiResult};
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// 简化角色配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimplifiedRoleConfig {
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 推荐模型
    pub recommended_model: String,
    /// 推荐模型列表
    pub recommended_models: Vec<String>,
    /// 核心能力
    pub capabilities: Option<Vec<String>>,
    /// 工作流程步骤
    pub workflow: Option<Vec<String>>,
}

/// AI使用规则配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageRules {
    /// 使用场景
    pub usage_scenarios: Vec<String>,
    /// 特殊约束
    pub constraints: AiUsageConstraints,
    /// 输出要求
    pub output_requirements: AiOutputRequirements,
    /// 最佳实践
    pub best_practices: Vec<String>,
    /// 禁止行为
    pub prohibited_actions: Vec<String>,
}

/// AI使用约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageConstraints {
    /// 最大代码长度
    pub max_code_length: Option<u32>,
    /// 允许的语言
    pub allowed_languages: Option<Vec<String>>,
    /// 代码审查要求
    pub code_review_required: Option<bool>,
    /// 安全扫描要求
    pub security_scan_required: Option<bool>,
    /// 最大数据集大小
    pub max_dataset_size: Option<String>,
    /// 允许的数据格式
    pub allowed_data_formats: Option<Vec<String>>,
    /// 数据隐私级别
    pub data_privacy_level: Option<String>,
    /// 统计验证要求
    pub statistical_validation_required: Option<bool>,
    /// 最大内容长度
    pub max_content_length: Option<u32>,
    /// 允许的内容类型
    pub allowed_content_types: Option<Vec<String>>,
    /// 抄袭检查要求
    pub plagiarism_check_required: Option<bool>,
    /// 风格一致性要求
    pub style_consistency_required: Option<bool>,
    /// 最大日志大小
    pub max_log_size: Option<String>,
    /// 允许的调试级别
    pub allowed_debug_levels: Option<Vec<String>>,
    /// 系统访问级别
    pub system_access_level: Option<String>,
    /// 生产安全要求
    pub production_safety_required: Option<bool>,
    /// 每次会话最大审查文件数
    pub max_review_files_per_session: Option<u32>,
    /// 允许的审查类型
    pub allowed_review_types: Option<Vec<String>>,
    /// 审查深度级别
    pub review_depth_level: Option<String>,
    /// 合规性检查要求
    pub compliance_check_required: Option<bool>,
    /// 每次会话最大测试用例数
    pub max_test_cases_per_session: Option<u32>,
    /// 允许的测试类型
    pub allowed_test_types: Option<Vec<String>>,
    /// 测试数据要求
    pub test_data_requirements: Option<String>,
}

/// AI输出要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiOutputRequirements {
    /// 输出格式
    pub format: String,
    /// 包含指标
    pub include_metrics: Option<bool>,
    /// 错误处理方式
    pub error_handling: String,
    /// 包含注释
    pub include_comments: Option<bool>,
    /// 包含测试
    pub include_tests: Option<bool>,
    /// 包含可视化
    pub include_visualizations: Option<bool>,
    /// 包含置信区间
    pub include_confidence_intervals: Option<bool>,
    /// 语法检查
    pub grammar_check: Option<bool>,
    /// 包含引用
    pub include_references: Option<bool>,
    /// 包含根本原因分析
    pub include_root_cause_analysis: Option<bool>,
    /// 包含修复建议
    pub include_fix_suggestions: Option<bool>,
    /// 包含严重性评级
    pub include_severity_ratings: Option<bool>,
    /// 包含行动项
    pub include_action_items: Option<bool>,
}

/// 全局AI规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalAiRules {
    /// 基础原则
    pub principles: Vec<String>,
    /// 通用约束
    pub constraints: GlobalConstraints,
    /// 质量标准
    pub quality_standards: QualityStandards,
    /// 安全要求
    pub security_requirements: SecurityRequirements,
}

/// 全局约束
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConstraints {
    /// 每个请求最大令牌数
    pub max_tokens_per_request: u32,
    /// 每分钟最大请求数
    pub max_requests_per_minute: u32,
    /// 最大并发请求数
    pub max_concurrent_requests: u32,
    /// 允许的内容类型
    pub allowed_content_types: Vec<String>,
    /// 禁止的内容
    pub forbidden_content: Vec<String>,
}

/// 质量标准
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStandards {
    /// 最小响应质量
    pub min_response_quality: f32,
    /// 需要审查
    pub required_review: bool,
    /// 验证步骤
    pub validation_steps: Vec<String>,
}

/// 安全要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRequirements {
    /// 数据加密
    pub data_encryption: bool,
    /// 审计日志
    pub audit_logging: bool,
    /// 访问控制
    pub access_control: bool,
    /// 内容过滤
    pub content_filtering: bool,
}

/// AI使用规则配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageRulesConfig {
    /// 全局AI使用规则
    pub global_ai_rules: GlobalAiRules,
    /// 角色特定的AI使用规则
    pub role_specific_rules: HashMap<String, AiUsageRules>,
    /// AI使用工作流程
    pub ai_usage_workflows: HashMap<String, AiUsageWorkflow>,
    /// 监控和审计
    pub monitoring_and_audit: MonitoringAndAudit,
    /// 培训和支持
    pub training_and_support: TrainingAndSupport,
    /// 元数据
    pub metadata: Metadata,
}

/// AI使用工作流程
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiUsageWorkflow {
    /// 工作流程步骤
    pub steps: Vec<WorkflowStep>,
    /// 质量控制点
    pub quality_checkpoints: Vec<String>,
    /// 异常处理
    pub exception_handling: Vec<String>,
}

/// 工作流程步骤
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 是否必需
    pub required: bool,
}

/// 监控和审计
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringAndAudit {
    /// 使用指标
    pub usage_metrics: Vec<String>,
    /// 审计日志
    pub audit_log: AuditLogConfig,
    /// 性能监控
    pub performance_monitoring: PerformanceMonitoring,
}

/// 审计日志配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogConfig {
    /// 是否启用
    pub enabled: bool,
    /// 保留期
    pub retention_period: String,
    /// 日志级别
    pub log_level: String,
    /// 包含敏感数据
    pub include_sensitive_data: bool,
}

/// 性能监控
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMonitoring {
    /// 是否启用
    pub enabled: bool,
    /// 告警阈值
    pub alert_thresholds: HashMap<String, String>,
    /// 报告频率
    pub reporting_frequency: String,
}

/// 培训和支持
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingAndSupport {
    /// 用户培训
    pub user_training: UserTraining,
    /// 技术支持
    pub technical_support: TechnicalSupport,
    /// 文档资源
    pub documentation: Documentation,
}

/// 用户培训
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTraining {
    /// 是否必需
    pub required: bool,
    /// 培训模块
    pub training_modules: Vec<String>,
    /// 认证要求
    pub certification_required: bool,
    /// 刷新频率
    pub refresher_frequency: String,
}

/// 技术支持
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalSupport {
    /// 是否可用
    pub available: bool,
    /// 支持渠道
    pub support_channels: Vec<String>,
    /// 响应时间
    pub response_time: HashMap<String, String>,
}

/// 文档
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Documentation {
    /// 用户指南
    pub user_guide: String,
    /// API参考
    pub api_reference: String,
    /// 最佳实践
    pub best_practices: String,
    /// 故障排除
    pub troubleshooting: String,
}

/// 元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// 版本
    pub version: String,
    /// 最后更新时间
    pub last_updated: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 更新日志
    pub changelog: HashMap<String, String>,
    /// 审查周期
    pub review_cycle: Option<String>,
    /// 下次审查日期
    pub next_review_date: Option<String>,
}

/// 角色配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleConfig {
    /// 角色名称
    pub name: String,
    /// 角色描述
    pub description: String,
    /// 系统提示词
    pub system_prompt: String,
    /// 推荐模型
    pub recommended_model: String,
    /// 推荐模型列表
    pub recommended_models: Vec<String>,
}

/// 角色配置管理器
#[derive(Debug)]
pub struct RoleConfigManager {
    /// 角色配置映射
    pub roles: HashMap<String, RoleConfig>,
    /// 配置文件路径
    pub config_path: String,
}

impl RoleConfigManager {
    /// 保存角色配置到YAML文件
    pub fn save_config(&self) -> AiResult<()> {
        let path = Path::new(&self.config_path);

        let content = serde_yaml::to_string(&self.roles).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("序列化角色配置失败: {e}")))
        })?;

        fs::write(path, content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "写入角色配置文件失败: {e}"
            )))
        })?;

        Ok(())
    }
    
    /// 加载AI使用规则配置
    pub fn load_ai_usage_rules(&self) -> AiResult<AiUsageRulesConfig> {
        let ai_rules_path = Path::new(&self.config_path).with_file_name("ai_usage_rules.yaml");
        
        if !ai_rules_path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "AI usage rules file not found: {}",
                ai_rules_path.display()
            ))));
        }
        
        let content = fs::read_to_string(&ai_rules_path)
            .map_err(|e| AiError::from(AiErrReason::ConfigError(format!("Failed to read AI usage rules file: {}", e))))?;
        
        let config: AiUsageRulesConfig = serde_yaml::from_str(&content)
            .map_err(|e| AiError::from(AiErrReason::ConfigError(format!("Failed to parse AI usage rules: {}", e))))?;
        
        Ok(config)
    }
    
    /// 获取指定角色的AI使用规则
    pub fn get_role_ai_usage_rules(&self, role_name: &str) -> AiResult<AiUsageRules> {
        let ai_rules_config = self.load_ai_usage_rules()?;
        
        ai_rules_config.role_specific_rules.get(role_name)
            .cloned()
            .ok_or_else(|| AiError::from(AiErrReason::ConfigError(format!(
                "AI usage rules not found for role: {}",
                role_name
            ))))
    }
    
    /// 检查角色是否允许执行特定操作
    pub fn is_action_allowed(&self, role_name: &str, action: &str) -> AiResult<bool> {
        let rules = self.get_role_ai_usage_rules(role_name)?;
        
        Ok(!rules.prohibited_actions.contains(&action.to_string()))
    }
    
    /// 获取角色的输出要求
    pub fn get_role_output_requirements(&self, role_name: &str) -> AiResult<AiOutputRequirements> {
        let rules = self.get_role_ai_usage_rules(role_name)?;
        Ok(rules.output_requirements)
    }
    
    /// 获取全局AI规则
    pub fn get_global_ai_rules(&self) -> AiResult<GlobalAiRules> {
        let ai_rules_config = self.load_ai_usage_rules()?;
        Ok(ai_rules_config.global_ai_rules)
    }
    
    /// 验证AI使用是否符合规则
    pub fn validate_ai_usage(&self, role_name: &str, action: &str, content: &str) -> AiResult<()> {
        // 检查全局约束
        let global_rules = self.get_global_ai_rules()?;
        
        // 检查内容长度
        if content.len() > global_rules.constraints.max_tokens_per_request as usize {
            return Err(AiError::from(AiErrReason::ConfigError(
                "Content exceeds maximum token limit".to_string()
            )));
        }
        
        // 检查禁止内容
        for forbidden in &global_rules.constraints.forbidden_content {
            if content.contains(forbidden) {
                return Err(AiError::from(AiErrReason::ConfigError(
                    format!("Content contains forbidden material: {}", forbidden)
                )));
            }
        }
        
        // 检查角色特定规则
        let role_rules = self.get_role_ai_usage_rules(role_name)?;
        
        // 检查禁止行为
        if role_rules.prohibited_actions.contains(&action.to_string()) {
            return Err(AiError::from(AiErrReason::ConfigError(
                format!("Action '{}' is prohibited for role '{}'", action, role_name)
            )));
        }
        
        // 检查最大代码长度（如果适用）
        if let Some(max_code_length) = role_rules.constraints.max_code_length {
            if content.len() > max_code_length as usize {
                return Err(AiError::from(AiErrReason::ConfigError(
                    "Code exceeds maximum length limit".to_string()
                )));
            }
        }
        
        Ok(())
    }
    
    /// 获取角色的最佳实践建议
    pub fn get_role_best_practices(&self, role_name: &str) -> AiResult<Vec<String>> {
        let rules = self.get_role_ai_usage_rules(role_name)?;
        Ok(rules.best_practices)
    }
    
    /// 获取角色的使用场景
    pub fn get_role_usage_scenarios(&self, role_name: &str) -> AiResult<Vec<String>> {
        let rules = self.get_role_ai_usage_rules(role_name)?;
        Ok(rules.usage_scenarios)
    }

    /// 创建默认的角色配置YAML文件
    pub fn create_default_config(config_path: &str) -> AiResult<()> {
        let default_roles: HashMap<String, RoleConfig> = HashMap::new();
        let path = Path::new(config_path);

        let content = serde_yaml::to_string(&default_roles).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "序列化默认角色配置失败: {e}"
            )))
        })?;

        fs::write(path, content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "创建默认角色配置文件失败: {e}"
            )))
        })?;

        Ok(())
    }

    /// 验证YAML配置文件格式
    pub fn validate_config_file(config_path: &str) -> AiResult<()> {
        let path = Path::new(config_path);

        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "配置文件不存在: {}",
                config_path
            ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("读取配置文件失败: {e}")))
        })?;

        serde_yaml::from_str::<HashMap<String, RoleConfig>>(&content).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!("YAML格式验证失败: {e}")))
        })?;

        Ok(())
    }
    /// 创建新的角色配置管理器
    pub fn new(config_path: String) -> Self {
        Self {
            roles: HashMap::new(),
            config_path,
        }
    }

    /// 从文件加载角色配置
    pub fn load_config(&mut self) -> AiResult<()> {
        let path = Path::new(&self.config_path);

        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "角色配置文件不存在: {}",
                self.config_path
            ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "读取角色配置文件失败: {e}"
            )))
        })?;

        let roles_config: HashMap<String, RoleConfig> =
            serde_yaml::from_str(&content).map_err(|e| {
                AiError::from(AiErrReason::ConfigError(format!(
                    "解析角色配置文件失败: {e}"
                )))
            })?;

        self.roles = roles_config;
        Ok(())
    }

    /// 获取角色配置
    pub fn get_role_config(&self, role_key: &str) -> Option<&RoleConfig> {
        self.roles.get(role_key)
    }

    /// 获取所有可用的角色
    pub fn get_available_roles(&self) -> Vec<&String> {
        self.roles.keys().collect()
    }

    /// 重新加载配置
    pub fn reload_config(&mut self) -> AiResult<()> {
        self.roles.clear();
        self.load_config()
    }

    /// 检查角色是否存在
    pub fn role_exists(&self, role_key: &str) -> bool {
        self.roles.contains_key(role_key)
    }

    /// 加载简化角色配置
    pub fn load_simplified_config(&mut self, config_path: &str) -> AiResult<()> {
        let path = Path::new(config_path);

        if !path.exists() {
            return Err(AiError::from(AiErrReason::ConfigError(format!(
                "简化角色配置文件不存在: {}",
                config_path
            ))));
        }

        let content = fs::read_to_string(path).map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "读取简化角色配置文件失败: {e}"
            )))
        })?;

        let simplified_roles: HashMap<String, SimplifiedRoleConfig> =
            serde_yaml::from_str(&content).map_err(|e| {
                AiError::from(AiErrReason::ConfigError(format!(
                    "解析简化角色配置文件失败: {e}"
                )))
            })?;

        // 转换为标准RoleConfig格式
        for (key, simplified_role) in simplified_roles {
            let role_config = RoleConfig {
                name: simplified_role.name,
                description: simplified_role.description,
                system_prompt: simplified_role.system_prompt,
                recommended_model: simplified_role.recommended_model,
                recommended_models: simplified_role.recommended_models,
            };
            self.roles.insert(key, role_config);
        }

        Ok(())
    }

    /// 自动检测并加载配置（优先尝试简化配置）
    pub fn auto_load_config(&mut self, simplified_path: &str, legacy_path: &str) -> AiResult<()> {
        // 首先尝试加载简化配置
        if Path::new(simplified_path).exists() {
            println!("📄 Loading simplified roles configuration...");
            return self.load_simplified_config(simplified_path);
        }
        
        // 回退到传统配置
        if Path::new(legacy_path).exists() {
            println!("📄 Loading legacy roles configuration...");
            return self.load_config();
        }
        
        Err(AiError::from(AiErrReason::ConfigError(
            "Neither simplified nor legacy configuration file found".to_string()
        )))
    }
}

impl Default for RoleConfigManager {
    fn default() -> Self {
        Self::new("src/ai/config/roles.yaml".to_string())
    }
}

/// 角色配置加载器
pub struct RoleConfigLoader;

impl Default for RoleConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl RoleConfigLoader {
    /// 创建新的角色配置加载器
    pub fn new() -> Self {
        Self
    }

    /// 创建并加载角色配置管理器
    pub fn load(config_path: Option<String>) -> AiResult<RoleConfigManager> {
        let path = config_path.unwrap_or_else(|| "src/ai/config/roles.json".to_string());
        let mut manager = RoleConfigManager::new(path);
        manager.load_config()?;
        Ok(manager)
    }

    /// 创建默认角色配置管理器
    pub fn load_default() -> AiResult<RoleConfigManager> {
        Self::load(None)
    }

    /// 加载简化角色配置管理器
    pub fn load_simplified(config_path: Option<String>) -> AiResult<RoleConfigManager> {
        let path = config_path.unwrap_or_else(|| "src/ai/config/roles_simplified.yaml".to_string());
        let mut manager = RoleConfigManager::new(path.clone());
        manager.load_simplified_config(&path)?;
        Ok(manager)
    }

    /// 自动加载角色配置管理器（优先简化配置）
    pub fn auto_load(simplified_path: Option<String>, legacy_path: Option<String>) -> AiResult<RoleConfigManager> {
        let simplified = simplified_path.unwrap_or_else(|| "src/ai/config/roles_simplified.yaml".to_string());
        let legacy = legacy_path.unwrap_or_else(|| "src/ai/config/roles.yaml".to_string());
        let mut manager = RoleConfigManager::new(legacy.clone()); // 使用legacy作为默认路径
        manager.auto_load_config(&simplified, &legacy)?;
        Ok(manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_role_config_manager() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path);
        assert!(manager.load_config().is_ok());

        let role_config = manager.get_role_config("test_role");
        assert!(role_config.is_some());

        let config = role_config.unwrap();
        assert_eq!(config.name, "测试角色");
        assert_eq!(config.description, "这是一个测试角色");
        assert_eq!(config.system_prompt, "你是一个测试角色");
        assert_eq!(config.recommended_model, "test-model");
        assert_eq!(
            config.recommended_models,
            vec!["test-model", "backup-model"]
        );
    }

    #[test]
    fn test_role_config_manager_yaml() {
        let test_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "dev-model"
  recommended_models:
    - "dev-model"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.role_exists("developer"));
    }

    #[test]
    fn test_save_config() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path.clone());
        assert!(manager.load_config().is_ok());

        // Test saving config
        assert!(manager.save_config().is_ok());

        // Reload and verify
        let mut new_manager = RoleConfigManager::new(temp_path);
        assert!(new_manager.load_config().is_ok());
        assert!(new_manager.role_exists("test_role"));
    }

    #[test]
    fn test_role_config_loader_with_yaml() {
        let test_config = r#"
    analyst:
      name: "分析师"
      description: "专注于数据分析和洞察"
      system_prompt: "你是一个数据分析师，擅长从数据中发现规律和趋势"
      recommended_model: "gpt-4"
      recommended_models:
        - "gpt-4"
        - "claude-3-sonnet"
        - "deepseek-chat"
            "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path));
        assert!(manager.is_ok());

        let manager = manager.unwrap();
        assert!(manager.role_exists("analyst"));

        let role_config = manager.get_role_config("analyst");
        assert!(role_config.is_some());
        let config = role_config.unwrap();
        assert_eq!(config.name, "分析师");
        assert_eq!(config.recommended_model, "gpt-4");
        assert_eq!(
            config.recommended_models,
            vec!["gpt-4", "claude-3-sonnet", "deepseek-chat"]
        );
    }

    #[test]
    fn test_yaml_complex_roles() {
        let complex_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发和工程化"
  system_prompt: "你是一个资深的开发者，擅长编写高质量、可维护的代码"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3-sonnet"
    - "deepseek-coder"

writer:
  name: "作家"
  description: "专注于创意写作和文档生成"
  system_prompt: "你是一个专业的作家，擅长创作各种类型的文本内容"
  recommended_model: "claude-3-opus"
  recommended_models:
    - "claude-3-opus"
    - "gpt-4"
    - "deepseek-chat"

debugger:
  name: "调试专家"
  description: "专注于代码调试和问题排查"
  system_prompt: "你是一个资深的调试专家，擅长发现和解决代码中的各种问题"
  recommended_model: "deepseek-coder"
  recommended_models:
    - "deepseek-coder"
    - "gpt-4"
    - "claude-3-sonnet"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(complex_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path)).unwrap();

        // Test multiple roles
        assert!(manager.role_exists("developer"));
        assert!(manager.role_exists("writer"));
        assert!(manager.role_exists("debugger"));

        let dev_config = manager.get_role_config("developer").unwrap();
        assert_eq!(dev_config.name, "开发者");
        assert_eq!(dev_config.recommended_model, "gpt-4");
        assert_eq!(dev_config.recommended_models.len(), 3);

        let writer_config = manager.get_role_config("writer").unwrap();
        assert_eq!(writer_config.name, "作家");
        assert_eq!(writer_config.recommended_model, "claude-3-opus");

        let debugger_config = manager.get_role_config("debugger").unwrap();
        assert_eq!(debugger_config.name, "调试专家");
        assert_eq!(debugger_config.recommended_model, "deepseek-coder");
    }

    #[test]
    fn test_yaml_with_comments() {
        let config_with_comments = r#"# 这是角色配置文件的示例
# 包含多个AI角色定义

developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3"

# 这是分析师角色
analyst:
  name: "分析师"
  description: "数据分析"
  system_prompt: "你是一个分析师"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file
            .write_all(config_with_comments.as_bytes())
            .unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let manager = RoleConfigLoader::load(Some(temp_path)).unwrap();

        assert!(manager.role_exists("developer"));
        assert!(manager.role_exists("analyst"));

        let dev_config = manager.get_role_config("developer").unwrap();
        assert_eq!(dev_config.name, "开发者");
        assert_eq!(dev_config.recommended_models, vec!["gpt-4", "claude-3"]);
    }

    #[test]
    fn test_create_and_validate_default_config() {
        let temp_path = "/tmp/test_roles_default.yaml";

        // Test creating default config
        let result = RoleConfigManager::create_default_config(temp_path);
        assert!(result.is_ok());

        // Test validating the created config
        let validate_result = RoleConfigManager::validate_config_file(temp_path);
        assert!(validate_result.is_ok());

        // Test loading the created config
        let mut manager = RoleConfigManager::new(temp_path.to_string());
        assert!(manager.load_config().is_ok());

        // Should have no roles in default config
        let available_roles = manager.get_available_roles();
        assert_eq!(available_roles.len(), 0);

        // Clean up
        std::fs::remove_file(temp_path).unwrap();
    }

    #[test]
    fn test_invalid_yaml_format() {
        let invalid_config = r#"
developer:
  name: "开发者"
  description: "专注于代码开发"
  system_prompt: "你是一个开发者"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3"
  invalid_field: "this should cause an error"

analyst:
  name: "分析师"
  description: "数据分析"
        "#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(invalid_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap().to_string();

        let mut manager = RoleConfigManager::new(temp_path);
        let result = manager.load_config();

        // Should fail to parse due to invalid field
        assert!(result.is_err());
    }

    #[test]
    fn test_save_and_reload_cycle() {
        let test_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
        "#;

        let _temp_path = "/tmp/test_roles_save_reload.yaml";

        // Create initial manager
        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(test_config.as_bytes()).unwrap();
        let original_path = temp_file.path().to_str().unwrap().to_string();

        // Load original config
        let mut manager = RoleConfigManager::new(original_path.clone());
        assert!(manager.load_config().is_ok());

        // Modify the config
        let available_roles = manager.get_available_roles();
        assert_eq!(available_roles.len(), 1);

        // Save config
        assert!(manager.save_config().is_ok());

        // Create new manager and reload
        let mut new_manager = RoleConfigManager::new(original_path.clone());
        assert!(new_manager.load_config().is_ok());

        // Verify data integrity
        assert!(new_manager.role_exists("test_role"));
        let reloaded_config = new_manager.get_role_config("test_role").unwrap();
        assert_eq!(reloaded_config.name, "测试角色");
        assert_eq!(
            reloaded_config.recommended_models,
            vec!["test-model", "backup-model"]
        );

        // Clean up
        std::fs::remove_file(original_path.clone()).unwrap();
    }
}
