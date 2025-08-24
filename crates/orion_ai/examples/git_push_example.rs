use orion_ai::config::{ProviderConfig, RoutingRules, ThreadConfig, UsageLimits};
use orion_ai::func::git::{create_git_functions, GitFunctionExecutor};
use orion_ai::provider::{AiProviderType, AiRequest};
use orion_ai::{AiClient, AiConfig, FunctionExecutor, FunctionRegistry};
use std::collections::HashMap;

// 创建 Git 函数定义

#[tokio::main]
async fn main() -> orion_ai::AiResult<()> {
    // 1. 创建模拟配置进行测试
    println!("=== 使用 MockProvider 测试 Function Calling ===");

    // 创建只包含 MockProvider 的配置
    let mut providers = HashMap::new();
    providers.insert(
        AiProviderType::Mock,
        orion_ai::config::ProviderConfig {
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

    // 2. 创建客户端
    let client = AiClient::new(config.clone(), None)?;

    // 3. 创建函数注册表
    let mut registry = FunctionRegistry::new();

    // 4. 注册Git函数
    let git_functions = create_git_functions();
    for function in git_functions {
        registry.register_function(function)?;
    }

    // 5. 为每个Git函数注册执行器
    let git_executor = std::sync::Arc::new(GitFunctionExecutor);
    for function_name in git_executor.supported_functions() {
        registry.register_executor(function_name, git_executor.clone())?;
    }

    // 6. 首先测试简单的函数调用
    println!("=== 测试1: 简单的 Git 状态查询 ===");
    let request1 = AiRequest::builder()
        .model("mock-gpt")
        .system_prompt(
            "你是一个Git助手。当用户要求检查Git状态时，你必须调用git_status函数。".to_string(),
        )
        .user_prompt("git_status 请检查当前Git状态".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("发送简单Git状态请求...");
    println!(
        "注册的函数: {:?}",
        registry
            .get_functions()
            .iter()
            .map(|f| &f.name)
            .collect::<Vec<_>>()
    );

    let response1 = client
        .send_request_with_functions(request1, &registry)
        .await?;
    println!("响应内容: {}", response1.content);
    println!("响应函数调用: {:?}", response1.function_calls);

    // 6. 如果第一个测试成功，测试完整流程
    if response1.function_calls.is_some() {
        println!("\n=== 测试2: 完整Git工作流 ===");
        let request2 = AiRequest::builder()
            .model("mock-gpt") // 使用 MockProvider
            .system_prompt("你是一个Git助手。当用户要求执行Git操作时，你必须按顺序调用相应的函数：git_status -> git_add -> git_commit -> git_push".to_string())
            .user_prompt("git_status git_add git_commit git_push 请执行完整的Git工作流".to_string())
            .functions(create_git_functions())
            .enable_function_calling(true)
            .build();

        let response2 = client
            .send_request_with_functions(request2, &registry)
            .await?;
        println!("响应内容: {}", response2.content);
        println!("响应函数调用: {:?}", response2.function_calls);

        // 7. 处理函数调用
        if let Some(function_calls) = &response2.function_calls {
            println!("\n执行函数调用...");
            let final_result = client.handle_function_calls(&response2, &registry).await?;
            println!("\n最终结果:");
            println!("{}", final_result);
        }
    } else {
        println!("\n⚠️  警告: 第一个测试没有返回函数调用，跳过完整流程测试");
        println!("这表明当前提供商可能不支持 function calling");
        println!("建议尝试使用支持 function calling 的提供商（如 OpenAI）");
    }

    // 9. 如果没有函数调用，提供调试信息
    if response1.function_calls.is_none() {
        println!("\n=== 调试信息 ===");
        println!("当前模型: mock-gpt");
        println!("提供商配置: {:?}", config.providers.keys());
        println!("路由配置: {:?}", config.routing);
        println!("注册的函数数量: {}", registry.get_functions().len());

        // 输出函数定义用于调试
        println!("\n函数定义:");
        for func in registry.get_functions() {
            println!("- {}: {}", func.name, func.description);
            for param in &func.parameters {
                println!(
                    "  参数 {}: {} ({})",
                    param.name, param.description, param.r#type
                );
            }
        }

        println!("可能的原因:");
        println!("1. MockProvider 的 function calling 实现有问题");
        println!("2. 函数定义格式不正确");
        println!("3. FunctionRegistry 没有正确注册函数");
        println!("4. AiClient 的 function calling 逻辑有错误");
    }

    Ok(())
}
