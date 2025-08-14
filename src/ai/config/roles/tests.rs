use crate::ai::config::roles::loader::RoleConfigLoader;
use crate::ai::config::roles::manager::RoleConfigManager;
use orion_common::serde::Yamlable;
use orion_error::TestAssert;
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
    let temp_path = temp_file.path();

    let manager = RoleConfigManager::from_yml(&temp_path).assert();

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
