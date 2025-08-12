use tempfile::NamedTempFile;
use crate::ai::config::RoleConfigLoader;
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_simplified_config_loading() {
        // 创建临时简化配置文件
        let simplified_config = r#"
test_role:
  name: "测试角色"
  description: "这是一个测试角色"
  system_prompt: "你是一个测试角色"
  recommended_model: "test-model"
  recommended_models:
    - "test-model"
    - "backup-model"
  capabilities:
    - "testing"
    - "debugging"
  workflow:
    - "分析"
    - "实现"
    - "测试"

developer:
  name: "开发者"
  description: "专注于代码开发的技术专家"
  system_prompt: "你是一个专业的开发者，擅长高质量的代码实现、系统设计和技术问题解决。"
  recommended_model: "gpt-4"
  recommended_models:
    - "gpt-4"
    - "claude-3-sonnet"
    - "deepseek-coder"
  capabilities:
    - "coding"
    - "testing"
    - "debugging"
  workflow:
    - "需求分析"
    - "代码实现"
    - "测试验证"
"#;

        let mut temp_file = NamedTempFile::new().unwrap();
        temp_file.write_all(simplified_config.as_bytes()).unwrap();
        let temp_path = temp_file.path().to_str().unwrap();

        // 测试加载简化配置
        let result = RoleConfigLoader::load_simplified(Some(temp_path.to_string()));
        assert!(result.is_ok());
        
        let manager = result.unwrap();
        assert_eq!(manager.roles.len(), 2);
        
        // 验证角色配置
        let test_role = manager.get_role_config("test_role").unwrap();
        assert_eq!(test_role.name, "测试角色");
        assert_eq!(test_role.recommended_model, "test-model");
        assert_eq!(test_role.recommended_models.len(), 2);
        
        let developer_role = manager.get_role_config("developer").unwrap();
        assert_eq!(developer_role.name, "开发者");
        assert_eq!(developer_role.recommended_model, "gpt-4");
        assert_eq!(developer_role.recommended_models.len(), 3);
    }

    #[test]
    fn test_auto_load_prefers_simplified() {
        // 创建临时简化配置文件
        let simplified_config = r#"
test_role:
  name: "简化测试角色"
  description: "这是简化配置的测试角色"
  system_prompt: "你是一个简化测试角色"
  recommended_model: "simplified-model"
  recommended_models:
    - "simplified-model"
"#;

        // 创建临时传统配置文件
        let legacy_config = r#"
test_role:
  name: "传统测试角色"
  description: "这是传统配置的测试角色"
  system_prompt: "你是一个传统测试角色"
  recommended_model: "legacy-model"
  recommended_models:
    - "legacy-model"
"#;

        let mut simplified_file = NamedTempFile::new().unwrap();
        simplified_file.write_all(simplified_config.as_bytes()).unwrap();
        let simplified_path = simplified_file.path().to_str().unwrap();

        let mut legacy_file = NamedTempFile::new().unwrap();
        legacy_file.write_all(legacy_config.as_bytes()).unwrap();
        let legacy_path = legacy_file.path().to_str().unwrap();

        // 测试自动加载（应该优先使用简化配置）
        let result = RoleConfigLoader::auto_load(
            Some(simplified_path.to_string()),
            Some(legacy_path.to_string())
        );
        
        assert!(result.is_ok());
        let manager = result.unwrap();
        
        let role = manager.get_role_config("test_role").unwrap();
        // 应该加载简化配置的内容
        assert_eq!(role.name, "简化测试角色");
        assert_eq!(role.recommended_model, "simplified-model");
    }

    #[test]
    fn test_fallback_to_legacy() {
        // 只创建传统配置文件
        let legacy_config = r#"
test_role:
  name: "传统测试角色"
  description: "这是传统配置的测试角色"
  system_prompt: "你是一个传统测试角色"
  recommended_model: "legacy-model"
  recommended_models:
    - "legacy-model"
"#;

        let mut legacy_file = NamedTempFile::new().unwrap();
        legacy_file.write_all(legacy_config.as_bytes()).unwrap();
        let legacy_path = legacy_file.path().to_str().unwrap();

        // 测试自动加载（应该回退到传统配置）
        let result = RoleConfigLoader::auto_load(
            Some("nonexistent_simplified.yaml".to_string()),
            Some(legacy_path.to_string())
        );
        
        assert!(result.is_ok());
        let manager = result.unwrap();
        
        let role = manager.get_role_config("test_role").unwrap();
        // 应该加载传统配置的内容
        assert_eq!(role.name, "传统测试角色");
        assert_eq!(role.recommended_model, "legacy-model");
    }
}