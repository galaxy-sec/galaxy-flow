use orion_ai::config::{ProviderConfig, RoutingRules, ThreadConfig, UsageLimits};
use orion_ai::func::git::{create_git_functions, GitFunctionExecutor};
use orion_ai::provider::{AiProviderType, AiRequest};
use orion_ai::{AiClient, AiConfig, FunctionExecutor, FunctionRegistry};
use std::collections::HashMap;

#[tokio::test]
async fn test_mock_provider_function_calling() -> orion_ai::AiResult<()> {
    // 创建只包含 MockProvider 的配置
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
        routing: RoutingRules {
            simple: "mock-gpt".to_string(),
            complex: "mock-gpt".to_string(),
            free: "mock-gpt".to_string(),
        },
        limits: UsageLimits::default(),
        thread: ThreadConfig::default(),
    };

    // 创建客户端
    let client = AiClient::new(config, None)?;

    // 创建函数注册表
    let mut registry = FunctionRegistry::new();

    // 注册Git函数
    let git_functions = create_git_functions();
    for function in git_functions {
        registry.register_function(function)?;
    }

    // 为每个Git函数注册执行器
    let git_executor = std::sync::Arc::new(GitFunctionExecutor);
    for function_name in git_executor.supported_functions() {
        registry.register_executor(function_name, git_executor.clone())?;
    }

    // 测试1: 简单的 Git 状态查询
    let request1 = AiRequest::builder()
        .model("mock-gpt")
        .system_prompt(
            "你是一个Git助手。当用户要求检查Git状态时，你必须调用git_status函数。".to_string(),
        )
        .user_prompt("git-status 请检查当前Git状态".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    let response1 = client
        .send_request_with_functions(request1, &registry)
        .await?;

    // 验证第一个测试的函数调用
    assert!(response1.tool_calls.is_some(), "第一个测试应该返回函数调用");

    if let Some(function_calls) = &response1.tool_calls {
        assert_eq!(function_calls.len(), 1, "应该只调用一个函数");
        assert_eq!(
            function_calls[0].function.name, "git-status",
            "应该调用 git_status 函数"
        );

        // 检查参数是否包含在 JSON 字符串中
        let args = serde_json::from_str::<std::collections::HashMap<String, serde_json::Value>>(
            &function_calls[0].function.arguments,
        )
        .unwrap_or_default();
        assert!(args.contains_key("path"), "应该包含 path 参数");
        println!("函数调用: {:?}", function_calls[0]);
    }

    // 测试2: 完整Git工作流
    let request2 = AiRequest::builder()
        .model("mock-gpt")
        .system_prompt(
            "你是一个Git助手。当用户要求执行Git操作时，你必须按顺序调用相应的函数：git_status -> git_add -> git_commit -> git_push".to_string(),
        )
        .user_prompt("git-status git_add git_commit git_push 请执行完整的Git工作流".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    let response2 = client
        .send_request_with_functions(request2, &registry)
        .await?;

    // 验证第二个测试的函数调用
    assert!(response2.tool_calls.is_some(), "第二个测试应该返回函数调用");

    // 处理函数调用
    if let Some(function_calls) = &response2.tool_calls {
        assert!(!function_calls.is_empty(), "应该有函数调用");

        let final_result = client.handle_function_calls(&response2, &registry).await?;

        // 验证最终结果包含函数执行信息
        assert!(
            final_result.contains("git-status result:"),
            "最终结果应该包含 git_status 执行结果"
        );
    }

    // 测试函数注册表功能
    assert_eq!(registry.get_functions().len(), 4, "应该注册了4个Git函数");

    let git_status_func = registry.get_function("git-status");
    assert!(git_status_func.is_some(), "应该能找到 git_status 函数");

    if let Some(func) = git_status_func {
        assert_eq!(func.name, "git-status");
        assert!(!func.description.is_empty());
        assert!(!func.parameters.is_empty());
    }

    // 测试执行器注册
    assert!(
        registry.supports_function("git-status"),
        "应该支持 git_status 函数"
    );

    let supported_functions = registry.get_supported_function_names();
    assert!(
        supported_functions.contains(&"git-status".to_string()),
        "支持的函数列表应该包含 git_status"
    );

    Ok(())
}

#[tokio::test]
async fn test_mock_provider_single_function_call() -> orion_ai::AiResult<()> {
    // 创建简化配置，只测试单个函数调用
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
        routing: RoutingRules {
            simple: "mock-gpt".to_string(),
            complex: "mock-gpt".to_string(),
            free: "mock-gpt".to_string(),
        },
        limits: UsageLimits::default(),
        thread: ThreadConfig::default(),
    };

    let client = AiClient::new(config, None)?;
    let mut registry = FunctionRegistry::new();

    // 只注册 git_status 函数
    let git_functions = create_git_functions();
    let git_status_func = git_functions
        .into_iter()
        .find(|f| f.name == "git-status")
        .unwrap();

    registry.register_function(git_status_func.clone())?;

    // 为 git_status 注册执行器
    let git_executor = std::sync::Arc::new(GitFunctionExecutor);
    registry.register_executor("git-status".to_string(), git_executor.clone())?;

    // 测试单个函数调用
    let request = AiRequest::builder()
        .model("mock-gpt")
        .system_prompt("你是一个Git助手。调用git_status函数检查状态。".to_string())
        .user_prompt("git-status 检查状态".to_string())
        .functions(vec![git_status_func])
        .enable_function_calling(true)
        .build();

    let response = client
        .send_request_with_functions(request, &registry)
        .await?;

    assert!(response.tool_calls.is_some(), "应该返回函数调用");

    if let Some(function_calls) = &response.tool_calls {
        assert_eq!(function_calls.len(), 1);
        assert_eq!(function_calls[0].function.name, "git-status");
    }

    Ok(())
}

#[tokio::test]
async fn test_function_registry_basic() -> orion_ai::AiResult<()> {
    let mut registry = FunctionRegistry::new();

    // 测试空注册表
    assert_eq!(registry.get_functions().len(), 0);
    assert!(!registry.supports_function("test"));

    // 创建测试函数
    let test_function = orion_ai::provider::FunctionDefinition {
        name: "test_function".to_string(),
        description: "测试函数".to_string(),
        parameters: vec![],
    };

    // 注册函数
    registry.register_function(test_function.clone())?;

    // 创建并注册测试执行器
    struct TestExecutor;

    #[async_trait::async_trait]
    impl orion_ai::FunctionExecutor for TestExecutor {
        async fn execute(
            &self,
            function_call: &orion_ai::FunctionCall,
        ) -> orion_ai::AiResult<orion_ai::FunctionResult> {
            Ok(orion_ai::FunctionResult {
                name: function_call.function.name.clone(),
                result: serde_json::json!({"success": true}),
                error: None,
            })
        }

        fn supported_functions(&self) -> Vec<String> {
            vec!["test_function".to_string()]
        }

        fn get_function_schema(&self, function_name: &str) -> Option<orion_ai::FunctionDefinition> {
            if function_name == "test_function" {
                Some(orion_ai::provider::FunctionDefinition {
                    name: "test_function".to_string(),
                    description: "测试函数".to_string(),
                    parameters: vec![],
                })
            } else {
                None
            }
        }
    }

    let test_executor = std::sync::Arc::new(TestExecutor);
    registry.register_executor("test_function".to_string(), test_executor)?;

    // 验证注册成功

    let retrieved_func = registry.get_function("test_function");
    assert!(retrieved_func.is_some());

    if let Some(func) = retrieved_func {
        assert_eq!(func.name, "test_function");
        assert_eq!(func.description, "测试函数");
    }

    Ok(())
}
