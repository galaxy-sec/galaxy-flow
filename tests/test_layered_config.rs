use galaxy_flow::ai::config::roles::{RoleConfigLoader, RoleConfigManager};
use std::fs;
use std::path::Path;
use tempfile::TempDir;

#[test]
fn test_layered_load_project_priority() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 创建项目级配置目录
    let project_dir = temp_path.join("_gal");
    fs::create_dir_all(&project_dir).unwrap();

    // 创建项目级角色配置文件
    let project_roles_content = r#"
test_role:
  name: "项目级测试角色"
  description: "这是一个项目级测试角色"
  system_prompt: "你是一个项目级测试角色"
  recommended_model: "project-model"
  recommended_models:
    - "project-model"
  rules_path: "ai-rules"
"#;

    let project_roles_path = project_dir.join("ai-roles.yaml");
    fs::write(&project_roles_path, project_roles_content).unwrap();

    // 创建项目级规则目录
    let project_rules_dir = project_dir.join("ai-rules");
    fs::create_dir_all(&project_rules_dir).unwrap();

    // 创建项目级规则文件
    let project_rules_content = r#"项目级规则1
项目级规则2
项目级规则3"#;

    let project_rules_path = project_rules_dir.join("test_rules.mdc");
    fs::write(&project_rules_path, project_rules_content).unwrap();

    // 切换到临时目录
    std::env::set_current_dir(temp_path).unwrap();

    // 测试分层加载
    let result = RoleConfigLoader::layered_load();
    assert!(result.is_ok(), "分层配置加载失败: {:?}", result.err());

    let manager = result.unwrap();

    // 测试获取角色配置
    let role_config = manager.get_role_config("test_role");
    assert!(role_config.is_some(), "角色配置获取失败");

    let role_config = role_config.unwrap();
    assert_eq!(role_config.name, "项目级测试角色");
    assert_eq!(role_config.recommended_model, "project-model");

    // 测试获取角色规则配置
    let rules_result = manager.get_role_rules_config("test_role");
    assert!(
        rules_result.is_ok(),
        "角色规则配置获取失败: {:?}",
        rules_result.err()
    );

    let rules_config = rules_result.unwrap();
    assert!(rules_config.is_some(), "角色规则配置为空");

    let rules_config = rules_config.unwrap();
    assert_eq!(rules_config.rules.len(), 3);
    assert_eq!(rules_config.rules[0], "项目级规则1");
    assert_eq!(rules_config.rules[1], "项目级规则2");
    assert_eq!(rules_config.rules[2], "项目级规则3");
}

#[test]
fn test_get_layered_rules_path() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 创建项目级规则目录
    let project_rules_dir = temp_path.join("_gal").join("ai-rules");
    fs::create_dir_all(&project_rules_dir).unwrap();

    // 切换到临时目录
    std::env::set_current_dir(temp_path).unwrap();

    // 测试获取分层规则路径
    let result = RoleConfigLoader::get_layered_rules_path("some/path");
    assert!(result.is_ok(), "获取分层规则路径失败: {:?}", result.err());

    let rules_path = result.unwrap();
    assert!(rules_path.contains("_gal/ai-rules"));
}

#[test]
fn test_layered_load_no_config() {
    // 创建临时目录
    let temp_dir = TempDir::new().unwrap();
    let temp_path = temp_dir.path();

    // 切换到临时目录（没有配置文件）
    std::env::set_current_dir(temp_path).unwrap();

    // 测试分层加载应该失败
    let result = RoleConfigLoader::layered_load();
    assert!(result.is_err(), "期望分层配置加载失败，但成功了");

    let error = result.err().unwrap();
    assert!(error.to_string().contains("未找到有效的角色配置文件"));
}
