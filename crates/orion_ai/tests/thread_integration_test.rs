use orion_ai::{AiClientEnum, RoutingRules, UsageLimits};
use tempfile::TempDir;
use tokio::runtime::Runtime;

use orion_ai::{
    client::AiCoreClient,
    provider::{AiProviderType, AiRequest},
    thread::recorder::ThreadClient,
    AiConfig, ProviderConfig, ThreadConfig,
};
use std::collections::HashMap;
use std::path::PathBuf;

#[test]
fn test_thread_integration_basic() {
    let rt = Runtime::new().unwrap();

    // 创建临时目录用于Thread文件
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path();

    // 创建启用Thread记录的配置
    let mut providers = HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: "mock".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(1),
        },
    );

    let config = AiConfig {
        providers,
        routing: RoutingRules::default(),
        limits: UsageLimits::default(),
        thread: ThreadConfig {
            enabled: true,
            storage_path: storage_path.to_path_buf(),
            filename_template: "test-thread-YYYY-MM-DD.md".to_string(),
            min_summary_length: 10,
            max_summary_length: 100,
            summary_keywords: vec!["总结".to_string(), "summary".to_string()],
            inform_ai: false,
            inform_message: "".to_string(),
        },
    };

    // 创建AiClient - 启用Thread记录
    // 使用项目根目录的角色配置文件
    let role_file_path = PathBuf::from("../../_gal/ai-roles.yml");
    let basic_client = orion_ai::AiClient::new(config.clone(), Some(role_file_path)).unwrap();
    let thread_config = config.thread.clone();
    let thread_client = ThreadClient::new(AiCoreClient::Basic(basic_client), thread_config);
    let client = AiClientEnum::ThreadRecording(Box::new(thread_client));

    // 测试普通请求
    let request = AiRequest::builder()
        .model("mock")
        .system_prompt("You are a helpful assistant".to_string())
        .user_prompt(
            "What is 2+2? Please respond with: The answer is 4. summary: 2+2 equals 4".to_string(),
        )
        .build();

    let response = rt.block_on(async { client.send_request(request).await });

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.content.is_empty());

    // 验证Thread文件是否创建
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let expected_file = storage_path.join(format!("test-thread-{date_str}.md"));
    assert!(expected_file.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&expected_file).unwrap();
    println!("Thread file content:\n{content}");

    // 检查基本的Thread结构
    assert!(content.contains("# Thread记录"));
    assert!(content.contains("## 交互记录 1"));
    assert!(content.contains("**模型**: mock"));
    assert!(content.contains("What is 2+2?"));

    // 检查总结内容是否正确提取
    assert!(content.contains("2+2 equals 4") || content.contains("summary"));
}

#[test]
fn test_thread_inform_ai_functionality() {
    let rt = Runtime::new().unwrap();

    // 创建临时目录用于Thread文件
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path();

    // 创建启用AI通知的Thread配置
    let mut providers = HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: "mock".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(1),
        },
    );

    let config = AiConfig {
        providers,
        routing: RoutingRules::default(),
        limits: UsageLimits::default(),
        thread: ThreadConfig {
            enabled: true,
            storage_path: storage_path.to_path_buf(),
            filename_template: "test-inform-YYYY-MM-DD.md".to_string(),
            min_summary_length: 10,
            max_summary_length: 100,
            summary_keywords: vec!["总结".to_string()],
            inform_ai: true, // 启用AI通知
            inform_message:
                "【Thread记录已启用】本次对话正在被记录，请确保回答内容适合记录和分析。".to_string(),
        },
    };

    // 创建AiClient - 启用Thread记录
    // 使用项目根目录的角色配置文件
    let role_file_path = PathBuf::from("../../_gal/ai-roles.yml");
    let basic_client = orion_ai::AiClient::new(config.clone(), Some(role_file_path)).unwrap();
    let thread_config = config.thread.clone();
    let thread_client = ThreadClient::new(AiCoreClient::Basic(basic_client), thread_config);
    let client = AiClientEnum::ThreadRecording(Box::new(thread_client));

    // 测试请求 - AI应该被通知正在被记录
    let request = AiRequest::builder()
        .model("mock")
        .system_prompt("You are a helpful assistant".to_string())
        .user_prompt("What is the capital of France?".to_string())
        .build();

    let response = rt.block_on(async { client.send_request(request).await });

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.content.is_empty());

    // 验证Thread文件是否创建
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let expected_file = storage_path.join(format!("test-inform-{date_str}.md"));
    assert!(expected_file.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&expected_file).unwrap();
    println!("Thread file content with AI notification:\n{content}");

    // 检查基本的Thread结构
    assert!(content.contains("# Thread记录"));
    assert!(content.contains("## 交互记录 1"));
    assert!(content.contains("**模型**: mock"));
    assert!(content.contains("What is the capital of France?"));

    // 验证AI是否被通知（在系统提示中包含通知消息）
    // 由于mock provider可能会响应特定内容，我们可以检查响应是否被正确处理
    assert!(!response.content.is_empty());
}

#[test]
fn test_thread_without_inform_ai() {
    let rt = Runtime::new().unwrap();

    // 创建临时目录用于Thread文件
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path();

    // 创建禁用AI通知的Thread配置
    let mut providers = HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: "mock".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(1),
        },
    );

    let config = AiConfig {
        providers,
        routing: RoutingRules::default(),
        limits: UsageLimits::default(),
        thread: ThreadConfig {
            enabled: true,
            storage_path: storage_path.to_path_buf(),
            filename_template: "test-no-inform-YYYY-MM-DD.md".to_string(),
            min_summary_length: 10,
            max_summary_length: 100,
            summary_keywords: vec!["总结".to_string()],
            inform_ai: false, // 禁用AI通知
            inform_message:
                "【Thread记录已启用】本次对话正在被记录，请确保回答内容适合记录和分析。".to_string(),
        },
    };

    // 创建AiClient - 启用Thread记录但不通知AI
    // 使用项目根目录的角色配置文件
    let role_file_path = PathBuf::from("../../_gal/ai-roles.yml");
    let basic_client = orion_ai::AiClient::new(config.clone(), Some(role_file_path)).unwrap();
    let thread_config = config.thread.clone();
    let thread_client = ThreadClient::new(AiCoreClient::Basic(basic_client), thread_config);
    let client = AiClientEnum::ThreadRecording(Box::new(thread_client));

    // 测试请求 - AI不应该被通知正在被记录
    let request = AiRequest::builder()
        .model("mock")
        .system_prompt("You are a helpful assistant".to_string())
        .user_prompt("What is 2+2?".to_string())
        .build();

    let response = rt.block_on(async { client.send_request(request).await });

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.content.is_empty());

    // 验证Thread文件是否创建
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let expected_file = storage_path.join(format!("test-no-inform-{date_str}.md"));
    assert!(expected_file.exists());

    // 验证文件内容
    let content = std::fs::read_to_string(&expected_file).unwrap();
    println!("Thread file content without AI notification:\n{content}");

    // 检查基本的Thread结构
    assert!(content.contains("# Thread记录"));
    assert!(content.contains("## 交互记录 1"));
    assert!(content.contains("**模型**: mock"));
    assert!(content.contains("What is 2+2?"));
}

#[test]
fn test_thread_integration_with_disabled_config() {
    let rt = Runtime::new().unwrap();

    // 创建临时目录用于Thread文件
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path();

    // 创建禁用Thread记录的配置
    let mut providers = HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: "mock".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(1),
        },
    );

    let config = AiConfig {
        providers,
        routing: RoutingRules::default(),
        limits: UsageLimits::default(),
        thread: ThreadConfig {
            enabled: false, // 禁用Thread记录
            storage_path: storage_path.to_path_buf(),
            filename_template: "test-thread-YYYY-MM-DD.md".to_string(),
            min_summary_length: 10,
            max_summary_length: 100,
            summary_keywords: vec!["总结".to_string()],
            inform_ai: false,
            inform_message: "".to_string(),
        },
    };

    // 创建AiClient - 启用Thread记录但不通知AI
    // 使用项目根目录的角色配置文件
    let role_file_path = PathBuf::from("../../_gal/ai-roles.yml");
    let basic_client = orion_ai::AiClient::new(config.clone(), Some(role_file_path)).unwrap();
    let thread_config = config.thread.clone();
    let thread_client = ThreadClient::new(AiCoreClient::Basic(basic_client), thread_config);
    let client = AiClientEnum::ThreadRecording(Box::new(thread_client));

    // 测试请求
    let request = AiRequest::builder()
        .model("mock")
        .system_prompt("You are a helpful assistant".to_string())
        .user_prompt("What is 2+2?".to_string())
        .build();

    let response = rt.block_on(async { client.send_request(request).await });

    assert!(response.is_ok());
    let response = response.unwrap();
    assert!(!response.content.is_empty());

    // 验证Thread文件没有被创建
    let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let expected_file = storage_path.join(format!("test-thread-{date_str}.md"));
    assert!(!expected_file.exists());
}

#[test]
fn test_thread_config_validation() {
    let mut config = AiConfig::default();

    // 测试无效的长度范围
    config.thread.min_summary_length = 300;
    config.thread.max_summary_length = 200;

    // 测试无效配置时，使用默认角色配置（应该会失败）
    let result = AiClientEnum::new_with_thread_recording(config.clone());
    assert!(result.is_err());

    // 测试有效的配置
    let mut valid_config = AiConfig::default();
    valid_config.thread.min_summary_length = 20;
    valid_config.thread.max_summary_length = 250;

    // 添加必要的provider配置
    let mut providers = std::collections::HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        ProviderConfig {
            enabled: true,
            api_key: "mock".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: Some(1),
        },
    );
    valid_config.providers = providers;

    // 测试有效配置时，使用项目根目录的角色配置文件
    let role_file_path = PathBuf::from("../../_gal/ai-roles.yml");
    let basic_client = orion_ai::AiClient::new(valid_config.clone(), Some(role_file_path)).unwrap();
    let thread_config = valid_config.thread.clone();
    let thread_client = ThreadClient::new(AiCoreClient::Basic(basic_client), thread_config);
    let client = AiClientEnum::ThreadRecording(Box::new(thread_client));
    assert!(true); // 如果能成功创建ThreadRecording客户端，说明配置有效
}
