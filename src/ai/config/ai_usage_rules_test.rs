use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    /// 测试加载AI使用规则配置
    #[test]
    fn test_load_ai_usage_rules() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        
        // 创建AI使用规则配置文件
        let ai_rules_path = temp_dir_path.join("ai_usage_rules.yaml");
        
        let ai_rules_content = r#"
global_ai_rules:
  principles:
    - "AI should be used ethically and responsibly"
    - "AI should enhance human capabilities, not replace them"
  constraints:
    max_tokens_per_request: 4000
    max_requests_per_minute: 60
    max_concurrent_requests: 5
    allowed_content_types:
      - "text/plain"
      - "text/markdown"
    forbidden_content:
      - "malicious_code"
      - "harmful_content"
  quality_standards:
    min_response_quality: 0.8
    required_review: true
    validation_steps:
      - "accuracy_check"
      - "safety_check"
  security_requirements:
    data_encryption: true
    audit_logging: true
    access_control: true
    content_filtering: true

role_specific_rules:
  developer:
    usage_scenarios:
      - "code_generation"
      - "code_review"
      - "debugging"
    constraints:
      max_code_length: 1000
      allowed_languages:
        - "rust"
        - "python"
        - "javascript"
      code_review_required: true
      security_scan_required: true
    output_requirements:
      format: "code"
      include_comments: true
      include_tests: true
      error_handling: "graceful"
    best_practices:
      - "Always review generated code before use"
      - "Follow security best practices"
      - "Test generated code thoroughly"
    prohibited_actions:
      - "generate_malicious_code"
      - "bypass_security_measures"
      - "ignore_error_handling"
  
  analyst:
    usage_scenarios:
      - "data_analysis"
      - "report_generation"
      - "insight_extraction"
    constraints:
      max_dataset_size: "1GB"
      allowed_data_formats:
        - "csv"
        - "json"
        - "parquet"
      data_privacy_level: "high"
      statistical_validation_required: true
    output_requirements:
      format: "report"
      include_metrics: true
      include_visualizations: true
      error_handling: "detailed"
    best_practices:
      - "Validate data quality before analysis"
      - "Document analysis methodology"
      - "Provide confidence intervals"
    prohibited_actions:
      - "share_sensitive_data"
      - "manipulate_results"
      - "ignore_data_limitations"

ai_usage_workflows:
  code_development:
    steps:
      - name: "requirement_analysis"
        description: "Analyze development requirements"
        required: true
      - name: "code_generation"
        description: "Generate code based on requirements"
        required: true
      - name: "code_review"
        description: "Review generated code for quality"
        required: true
    quality_checkpoints:
      - "code_quality_check"
      - "security_check"
      - "performance_check"
    exception_handling:
      - "handle_generation_errors"
      - "handle_security_violations"
  
  data_analysis:
    steps:
      - name: "data_validation"
        description: "Validate input data quality"
        required: true
      - name: "analysis_execution"
        description: "Execute data analysis"
        required: true
      - name: "result_validation"
        description: "Validate analysis results"
        required: true
    quality_checkpoints:
      - "data_quality_check"
      - "statistical_validity_check"
      - "result_accuracy_check"
    exception_handling:
      - "handle_data_quality_issues"
      - "handle_analysis_errors"

monitoring_and_audit:
  usage_metrics:
    - "request_count"
    - "response_time"
    - "error_rate"
    - "token_usage"
  audit_log:
    enabled: true
    retention_period: "90 days"
    log_level: "INFO"
    include_sensitive_data: false
  performance_monitoring:
    enabled: true
    alert_thresholds:
      error_rate: "5%"
      response_time: "5s"
    reporting_frequency: "daily"

training_and_support:
  user_training:
    required: true
    training_modules:
      - "ai_ethics"
      - "security_best_practices"
      - "effective_prompting"
    certification_required: true
    refresher_frequency: "6 months"
  technical_support:
    available: true
    support_channels:
      - "email"
      - "chat"
      - "documentation"
    response_time:
      critical: "1 hour"
      high: "4 hours"
      medium: "24 hours"
      low: "72 hours"
  documentation:
    user_guide: "/docs/ai_usage_guide.md"
    api_reference: "/docs/api_reference.md"
    best_practices: "/docs/best_practices.md"
    troubleshooting: "/docs/troubleshooting.md"

metadata:
  version: "1.0.0"
  last_updated: "2024-01-15T10:00:00Z"
  description: "AI Usage Rules Configuration"
  author: "AI Governance Team"
  changelog:
    "1.0.0": "Initial version with comprehensive AI usage rules"
  review_cycle: "quarterly"
  next_review_date: "2024-04-15"
"#;
        
        let mut file = std::fs::File::create(&ai_rules_path).unwrap();
        file.write_all(ai_rules_content.as_bytes()).unwrap();
        drop(file);
        
        // 创建RoleConfigManager实例（使用AI使用规则文件路径）
        let manager = RoleConfigManager::new(ai_rules_path.to_string_lossy().to_string());
        
        // 测试加载AI使用规则
        let result = manager.load_ai_usage_rules();
        assert!(result.is_ok(), "Failed to load AI usage rules: {:?}", result.err());
        
        let ai_rules_config = result.unwrap();
        
        // 验证全局规则
        assert_eq!(ai_rules_config.global_ai_rules.principles.len(), 2);
        assert_eq!(ai_rules_config.global_ai_rules.constraints.max_tokens_per_request, 4000);
        assert_eq!(ai_rules_config.global_ai_rules.quality_standards.min_response_quality, 0.8);
        
        // 验证角色特定规则
        assert!(ai_rules_config.role_specific_rules.contains_key("developer"));
        assert!(ai_rules_config.role_specific_rules.contains_key("analyst"));
        
        let developer_rules = &ai_rules_config.role_specific_rules["developer"];
        assert_eq!(developer_rules.usage_scenarios.len(), 3);
        assert_eq!(developer_rules.constraints.max_code_length, Some(1000));
        assert_eq!(developer_rules.output_requirements.format, "code");
        assert_eq!(developer_rules.best_practices.len(), 3);
        assert_eq!(developer_rules.prohibited_actions.len(), 3);
        
        // 清理临时目录
        temp_dir.close().unwrap();
    }
    
    /// 测试获取角色AI使用规则
    #[test]
    fn test_get_role_ai_usage_rules() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        
        // 创建AI使用规则配置文件
        let ai_rules_path = temp_dir_path.join("ai_usage_rules.yaml");
        
        let ai_rules_content = r#"
global_ai_rules:
  principles:
    - "AI should be used ethically"
  constraints:
    max_tokens_per_request: 4000
    max_requests_per_minute: 60
    max_concurrent_requests: 5
    allowed_content_types:
      - "text/plain"
    forbidden_content: []
  quality_standards:
    min_response_quality: 0.8
    required_review: true
    validation_steps: []
  security_requirements:
    data_encryption: true
    audit_logging: true
    access_control: true
    content_filtering: true

role_specific_rules:
  test_role:
    usage_scenarios:
      - "testing"
    constraints:
      max_code_length: 500
    output_requirements:
      format: "text"
      error_handling: "simple"
    best_practices:
      - "Test thoroughly"
    prohibited_actions:
      - "unsafe_operations"

ai_usage_workflows:
  default_workflow:
    steps:
      - name: "request_validation"
        description: "Validate the AI request"
        required: true
      - name: "ai_processing"
        description: "Process the request with AI"
        required: true
      - name: "response_validation"
        description: "Validate the AI response"
        required: true
    quality_checkpoints:
      - "pre_processing_check"
      - "post_processing_check"
    exception_handling:
      - "handle_invalid_input"
      - "handle_ai_errors"

monitoring_and_audit:
  usage_metrics:
    - "request_count"
    - "response_time"
    - "error_rate"
  audit_log:
    enabled: true
    retention_period: "90 days"
    log_level: "INFO"
    include_sensitive_data: false
  performance_monitoring:
    enabled: true
    alert_thresholds:
      response_time: "5000ms"
      error_rate: "5%"
    reporting_frequency: "daily"

training_and_support:
  user_training:
    required: true
    training_modules:
      - "ai_basics"
      - "ethical_usage"
    certification_required: false
    refresher_frequency: "6 months"
  technical_support:
    available: true
    support_channels:
      - "email"
      - "chat"
    response_time:
      low: "1 hour"
      medium: "4 hours"
      high: "24 hours"
  documentation:
    user_guide: "/docs/user_guide.md"
    api_reference: "/docs/api_reference.md"
    best_practices: "/docs/best_practices.md"
    troubleshooting: "/docs/troubleshooting.md"

metadata:
  version: "1.0.0"
  last_updated: "2024-01-01"
  description: "AI usage rules configuration"
  author: "System Administrator"
  changelog:
    "1.0.0": "Initial version"
  review_cycle: "quarterly"
  next_review_date: "2024-04-01"
"#;
        
        let mut file = std::fs::File::create(&ai_rules_path).unwrap();
        file.write_all(ai_rules_content.as_bytes()).unwrap();
        drop(file);
        
        // 创建RoleConfigManager实例（使用AI使用规则文件路径）
        let manager = RoleConfigManager::new(ai_rules_path.to_string_lossy().to_string());
        
        // 测试获取角色AI使用规则
        let result = manager.get_role_ai_usage_rules("test_role");
        assert!(result.is_ok(), "Failed to get role AI usage rules: {:?}", result.err());
        
        let role_rules = result.unwrap();
        assert_eq!(role_rules.usage_scenarios, vec!["testing".to_string()]);
        assert_eq!(role_rules.constraints.max_code_length, Some(500));
        assert_eq!(role_rules.output_requirements.format, "text");
        assert_eq!(role_rules.best_practices, vec!["Test thoroughly".to_string()]);
        assert_eq!(role_rules.prohibited_actions, vec!["unsafe_operations".to_string()]);
        
        // 测试不存在的角色
        let result = manager.get_role_ai_usage_rules("nonexistent_role");
        assert!(result.is_err());
        
        // 清理临时目录
        temp_dir.close().unwrap();
    }
    
    /// 测试操作权限检查
    #[test]
    fn test_is_action_allowed() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        
        // 创建AI使用规则配置文件
        let ai_rules_path = temp_dir_path.join("ai_usage_rules.yaml");
        
        let ai_rules_content = r#"
global_ai_rules:
  principles:
    - "AI should be used ethically"
  constraints:
    max_tokens_per_request: 4000
    max_requests_per_minute: 60
    max_concurrent_requests: 5
    allowed_content_types:
      - "text/plain"
    forbidden_content: []
  quality_standards:
    min_response_quality: 0.8
    required_review: true
    validation_steps: []
  security_requirements:
    data_encryption: true
    audit_logging: true
    access_control: true
    content_filtering: true

role_specific_rules:
  test_role:
    usage_scenarios:
      - "testing"
    constraints:
      max_code_length: 500
    output_requirements:
      format: "text"
      error_handling: "simple"
    best_practices:
      - "Test thoroughly"
    prohibited_actions:
      - "forbidden_action"
      - "dangerous_operation"

ai_usage_workflows:
  default_workflow:
    steps:
      - name: "request_validation"
        description: "Validate the AI request"
        required: true
      - name: "ai_processing"
        description: "Process the request with AI"
        required: true
      - name: "response_validation"
        description: "Validate the AI response"
        required: true
    quality_checkpoints:
      - "pre_processing_check"
      - "post_processing_check"
    exception_handling:
      - "handle_invalid_input"
      - "handle_ai_errors"

monitoring_and_audit:
  usage_metrics:
    - "request_count"
    - "response_time"
    - "error_rate"
  audit_log:
    enabled: true
    retention_period: "90 days"
    log_level: "INFO"
    include_sensitive_data: false
  performance_monitoring:
    enabled: true
    alert_thresholds:
      response_time: "5000ms"
      error_rate: "5%"
    reporting_frequency: "daily"

training_and_support:
  user_training:
    required: true
    training_modules:
      - "ai_basics"
      - "ethical_usage"
    certification_required: false
    refresher_frequency: "6 months"
  technical_support:
    available: true
    support_channels:
      - "email"
      - "chat"
    response_time:
      low: "1 hour"
      medium: "4 hours"
      high: "24 hours"
  documentation:
    user_guide: "/docs/user_guide.md"
    api_reference: "/docs/api_reference.md"
    best_practices: "/docs/best_practices.md"
    troubleshooting: "/docs/troubleshooting.md"

metadata:
  version: "1.0.0"
  last_updated: "2024-01-01"
  description: "AI usage rules configuration"
  author: "System Administrator"
  changelog:
    "1.0.0": "Initial version"
  review_cycle: "quarterly"
  next_review_date: "2024-04-01"
"#;
        
        let mut file = std::fs::File::create(&ai_rules_path).unwrap();
        file.write_all(ai_rules_content.as_bytes()).unwrap();
        drop(file);
        
        // 创建RoleConfigManager实例（使用AI使用规则文件路径）
        let manager = RoleConfigManager::new(ai_rules_path.to_string_lossy().to_string());
        
        // 测试允许的操作
        let result = manager.is_action_allowed("test_role", "safe_operation");
        assert!(result.is_ok());
        assert!(result.unwrap());
        
        // 测试禁止的操作
        let result = manager.is_action_allowed("test_role", "forbidden_action");
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // 测试另一个禁止的操作
        let result = manager.is_action_allowed("test_role", "dangerous_operation");
        assert!(result.is_ok());
        assert!(!result.unwrap());
        
        // 清理临时目录
        temp_dir.close().unwrap();
    }
    
    /// 测试AI使用验证
    #[test]
    fn test_validate_ai_usage() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        
        // 创建AI使用规则配置文件
        let ai_rules_path = temp_dir_path.join("ai_usage_rules.yaml");
        
        let ai_rules_content = r#"
global_ai_rules:
  principles:
    - "AI should be used ethically"
  constraints:
    max_tokens_per_request: 100
    max_requests_per_minute: 60
    max_concurrent_requests: 5
    allowed_content_types:
      - "text/plain"
    forbidden_content:
      - "malicious"
  quality_standards:
    min_response_quality: 0.8
    required_review: true
    validation_steps: []
  security_requirements:
    data_encryption: true
    audit_logging: true
    access_control: true
    content_filtering: true

role_specific_rules:
  test_role:
    usage_scenarios:
      - "testing"
    constraints:
      max_code_length: 50
    output_requirements:
      format: "text"
      error_handling: "simple"
    best_practices:
      - "Test thoroughly"
    prohibited_actions:
      - "forbidden_action"

ai_usage_workflows:
  default_workflow:
    steps:
      - name: "request_validation"
        description: "Validate the AI request"
        required: true
      - name: "ai_processing"
        description: "Process the request with AI"
        required: true
      - name: "response_validation"
        description: "Validate the AI response"
        required: true
    quality_checkpoints:
      - "pre_processing_check"
      - "post_processing_check"
    exception_handling:
      - "handle_invalid_input"
      - "handle_ai_errors"

monitoring_and_audit:
  usage_metrics:
    - "request_count"
    - "response_time"
    - "error_rate"
  audit_log:
    enabled: true
    retention_period: "90 days"
    log_level: "INFO"
    include_sensitive_data: false
  performance_monitoring:
    enabled: true
    alert_thresholds:
      response_time: "5000ms"
      error_rate: "5%"
    reporting_frequency: "daily"

training_and_support:
  user_training:
    required: true
    training_modules:
      - "ai_basics"
      - "ethical_usage"
    certification_required: false
    refresher_frequency: "6 months"
  technical_support:
    available: true
    support_channels:
      - "email"
      - "chat"
    response_time:
      low: "1 hour"
      medium: "4 hours"
      high: "24 hours"
  documentation:
    user_guide: "/docs/user_guide.md"
    api_reference: "/docs/api_reference.md"
    best_practices: "/docs/best_practices.md"
    troubleshooting: "/docs/troubleshooting.md"

metadata:
  version: "1.0.0"
  last_updated: "2024-01-01"
  description: "AI usage rules configuration"
  author: "System Administrator"
  changelog:
    "1.0.0": "Initial version"
  review_cycle: "quarterly"
  next_review_date: "2024-04-01"
"#;
        
        let mut file = std::fs::File::create(&ai_rules_path).unwrap();
        file.write_all(ai_rules_content.as_bytes()).unwrap();
        drop(file);
        
        // 创建RoleConfigManager实例（使用AI使用规则文件路径）
        let manager = RoleConfigManager::new(ai_rules_path.to_string_lossy().to_string());
        
        // 测试有效使用
        let result = manager.validate_ai_usage("test_role", "safe_action", "short content");
        assert!(result.is_ok());
        
        // 测试内容过长
        let long_content = "a".repeat(200);
        let result = manager.validate_ai_usage("test_role", "safe_action", &long_content);
        assert!(result.is_err());
        
        // 测试禁止内容
        let result = manager.validate_ai_usage("test_role", "safe_action", "This contains malicious content");
        assert!(result.is_err());
        
        // 测试禁止操作
        let result = manager.validate_ai_usage("test_role", "forbidden_action", "short content");
        assert!(result.is_err());
        
        // 测试代码长度限制
        let long_code = "fn main() {".to_string() + &"a".repeat(60) + "}";
        let result = manager.validate_ai_usage("test_role", "safe_action", &long_code);
        assert!(result.is_err());
        
        // 清理临时目录
        temp_dir.close().unwrap();
    }
    
    /// 测试获取最佳实践和使用场景
    #[test]
    fn test_get_best_practices_and_usage_scenarios() {
        // 创建临时目录
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_dir_path = temp_dir.path();
        
        // 创建AI使用规则配置文件
        let ai_rules_path = temp_dir_path.join("ai_usage_rules.yaml");
        
        let ai_rules_content = r#"
global_ai_rules:
  principles:
    - "AI should be used ethically"
  constraints:
    max_tokens_per_request: 4000
    max_requests_per_minute: 60
    max_concurrent_requests: 5
    allowed_content_types:
      - "text/plain"
    forbidden_content: []
  quality_standards:
    min_response_quality: 0.8
    required_review: true
    validation_steps: []
  security_requirements:
    data_encryption: true
    audit_logging: true
    access_control: true
    content_filtering: true

role_specific_rules:
  test_role:
    usage_scenarios:
      - "testing"
      - "development"
    constraints:
      max_code_length: 500
    output_requirements:
      format: "text"
      error_handling: "simple"
    best_practices:
      - "Test thoroughly"
      - "Follow coding standards"
    prohibited_actions:
      - "unsafe_operations"
      - "data_modification"

ai_usage_workflows:
  default_workflow:
    steps:
      - name: "request_validation"
        description: "Validate the AI request"
        required: true
      - name: "ai_processing"
        description: "Process the request with AI"
        required: true
      - name: "response_validation"
        description: "Validate the AI response"
        required: true
    quality_checkpoints:
      - "pre_processing_check"
      - "post_processing_check"
    exception_handling:
      - "handle_invalid_input"
      - "handle_ai_errors"

monitoring_and_audit:
  usage_metrics:
    - "request_count"
    - "response_time"
    - "error_rate"
  audit_log:
    enabled: true
    retention_period: "90 days"
    log_level: "INFO"
    include_sensitive_data: false
  performance_monitoring:
    enabled: true
    alert_thresholds:
      response_time: "5000ms"
      error_rate: "5%"
    reporting_frequency: "daily"

training_and_support:
  user_training:
    required: true
    training_modules:
      - "ai_basics"
      - "ethical_usage"
    certification_required: false
    refresher_frequency: "6 months"
  technical_support:
    available: true
    support_channels:
      - "email"
      - "chat"
    response_time:
      low: "1 hour"
      medium: "4 hours"
      high: "24 hours"
  documentation:
    user_guide: "/docs/user_guide.md"
    api_reference: "/docs/api_reference.md"
    best_practices: "/docs/best_practices.md"
    troubleshooting: "/docs/troubleshooting.md"

metadata:
  version: "1.0.0"
  last_updated: "2024-01-01"
  description: "AI usage rules configuration"
  author: "System Administrator"
  changelog:
    "1.0.0": "Initial version"
  review_cycle: "quarterly"
  next_review_date: "2024-04-01"
"#;
        
        let mut file = std::fs::File::create(&ai_rules_path).unwrap();
        file.write_all(ai_rules_content.as_bytes()).unwrap();
        drop(file);
        
        // 创建RoleConfigManager实例（使用AI使用规则文件路径）
        let manager = RoleConfigManager::new(ai_rules_path.to_string_lossy().to_string());
        
        // 测试获取最佳实践
        let result = manager.get_role_best_practices("test_role");
        assert!(result.is_ok());
        let best_practices = result.unwrap();
        assert_eq!(best_practices.len(), 2);
        assert!(best_practices.contains(&"Test thoroughly".to_string()));
        assert!(best_practices.contains(&"Follow coding standards".to_string()));
        
        // 测试获取使用场景
        let result = manager.get_role_usage_scenarios("test_role");
        assert!(result.is_ok());
        let usage_scenarios = result.unwrap();
        assert_eq!(usage_scenarios.len(), 2);
        assert!(usage_scenarios.contains(&"testing".to_string()));
        assert!(usage_scenarios.contains(&"development".to_string()));
        
        // 测试不存在的角色
        let result = manager.get_role_best_practices("nonexistent_role");
        assert!(result.is_err());
        
        let result = manager.get_role_usage_scenarios("nonexistent_role");
        assert!(result.is_err());
        
        // 清理临时目录
        temp_dir.close().unwrap();
    }
}