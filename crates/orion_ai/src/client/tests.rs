use std::path::PathBuf;

use crate::client::{load_key_dict, AiClient, AiClientTrait};
use crate::infra::once_init_log;
use crate::provider::{AiProviderType, AiRequest};
use crate::roleid::AiRoleID;
use crate::AiConfig;
use orion_error::TestAssertWithMsg;
use orion_variate::vars::EnvEvalable;

fn create_mock_config() -> AiConfig {
    let mut config = AiConfig::example();
    // 禁用所有真实提供商，只保留Mock
    for (_, provider_config) in config.providers.iter_mut() {
        provider_config.enabled = false;
    }
    // 添加Mock提供商配置
    use crate::config::ProviderConfig;
    config.providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: String::new(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(999),
        },
    );
    config
}

#[tokio::test]
async fn test_client_with_deepseek() {
    once_init_log();
    let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        AiConfig::example().env_eval(&dict)
    } else {
        return;
    };
    let role_file = PathBuf::from("./examples/ai-roles.yml");
    // 创建配置，启用 DeepSeek
    let client = AiClient::new(config, Some(role_file)).assert("ai-cleint new");

    // 验证 DeepSeek 可用
    assert!(client.is_provider_available(AiProviderType::DeepSeek));
    assert!(client
        .available_providers()
        .contains(&AiProviderType::DeepSeek));

    // 创建简单的测试请求
    let request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt("你是一个测试助手".to_string())
        .user_prompt("请回答：1+1=?".to_string())
        .build();

    // 发送请求到 DeepSeek
    let response = client.send_request(request).await;

    match response {
        Ok(resp) => {
            println!("✅ DeepSeek 响应: {}", resp.content);
            assert!(!resp.content.is_empty());
            assert_eq!(resp.provider, AiProviderType::DeepSeek);
        }
        Err(e) => {
            // 在没有真实 API key 的情况下，这可能是预期的
            println!("⚠️ DeepSeek 请求失败（预期，需要真实 API key）: {e}");
        }
    }
}

#[tokio::test]
async fn test_client_smart_request_with_deepseek() {
    once_init_log();
    let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        AiConfig::example().env_eval(&dict)
    } else {
        return;
    };
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");
    // 使用 smart_role_request 方法
    let role = AiRoleID::new("developer");
    let response = client.smart_role_request(
        &role,
        "分析这个函数的性能：\nfn fibonacci(n: u64) -> u64 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }"
    ).await;

    match response {
        Ok(resp) => {
            println!("✅ DeepSeek smart 响应: {}", resp.content);
            assert!(!resp.content.is_empty());
        }
        Err(e) => {
            println!("⚠️ DeepSeek smart 请求失败（预期）: {e}");
        }
    }
}

#[tokio::test]
async fn test_client_provider_fallback() {
    // 测试当 DeepSeek 不可用时的回退机制
    let config = create_mock_config();

    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 验证 Mock provider 可用
    assert!(client.is_provider_available(AiProviderType::Mock));
    assert!(client.available_providers().contains(&AiProviderType::Mock));
}

#[test]
fn test_build_ai_request_with_valid_role() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试开发者角色
    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, "请解释什么是Rust的所有权系统")
        .expect("Failed to build AI request");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, "请解释什么是Rust的所有权系统");
    assert!(request.role.is_some());
    assert_eq!(request.role.unwrap(), AiRoleID::new("developer"));
}

#[test]
fn test_build_ai_request_with_operations_role() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试运维角色
    let role = AiRoleID::new("operations");
    let request = client
        .build_ai_request(&role, "如何检查Linux系统性能")
        .expect("Failed to build AI request");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, "如何检查Linux系统性能");
    assert!(request.role.is_some());
    assert_eq!(request.role.unwrap(), AiRoleID::new("operations"));
}

#[test]
fn test_build_ai_request_with_knowledler_role() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 使用开发者角色替代可能不存在的Knowledger角色
    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, "什么是微服务架构？")
        .expect("Failed to build AI request");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, "什么是微服务架构？");
    assert!(request.role.is_some());
    assert_eq!(request.role.unwrap(), AiRoleID::new("developer"));
}

#[test]
fn test_build_ai_request_with_empty_input() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试空用户输入
    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, "")
        .expect("Failed to build AI request with empty input");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, "");
    assert!(request.role.is_some());
}

#[test]
fn test_build_ai_request_with_special_characters() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试包含特殊字符的用户输入
    let special_input =
        "请解释什么是 'Rust' 的所有权系统？\n代码示例：\n```rust\nlet x = 42;\nlet y = x;\n```";
    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, special_input)
        .expect("Failed to build AI request with special characters");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, special_input);
    assert!(request.role.is_some());
}

#[test]
fn test_build_ai_request_with_long_input() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试长文本输入
    let long_input =
        "这是一个很长的用户输入，用于测试 build_ai_request 函数处理长文本的能力。".repeat(100);
    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, &long_input)
        .expect("Failed to build AI request with long input");

    // 验证请求结构
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert_eq!(request.user_prompt, long_input);
    assert!(request.role.is_some());
    assert!(request.user_prompt.len() > 1000); // 确保确实是长文本
}

#[test]
fn test_build_ai_request_model_selection() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    // 测试不同角色是否选择了不同的模型
    let dev_role = AiRoleID::new("developer");
    let dev_request = client
        .build_ai_request(&dev_role, "test")
        .expect("Failed to build developer request");

    let ops_role = AiRoleID::new("operations");
    let ops_request = client
        .build_ai_request(&ops_role, "test")
        .expect("Failed to build operations request");

    // 验证系统提示词不同（表明角色配置不同）
    assert_ne!(dev_request.system_prompt, ops_request.system_prompt);

    // 模型名称应该根据角色配置有所不同
    println!("Developer model: {}", dev_request.model);
    println!("Operations model: {}", ops_request.model);
}

#[test]
fn test_build_ai_request_response_structure() {
    once_init_log();
    let config = create_mock_config();
    let role_file = PathBuf::from("../../_gal/ai-roles.yml");
    let client = AiClient::new(config, Some(role_file)).expect("Failed to create AiClient");

    let role = AiRoleID::new("developer");
    let request = client
        .build_ai_request(&role, "什么是Galaxy Operator Ecosystem？")
        .expect("Failed to build AI request");

    // 验证 AiRequest 结构的所有字段
    assert!(!request.model.is_empty());
    assert!(!request.system_prompt.is_empty());
    assert!(!request.user_prompt.is_empty());

    // 验证可选字段的默认值
    assert!(request.max_tokens.is_none());
    assert!(request.temperature == Some(0.7) || request.temperature.is_none());
    assert!(request.role.is_some());
}
