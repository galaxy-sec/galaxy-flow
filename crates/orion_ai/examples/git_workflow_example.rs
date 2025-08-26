use std::path::PathBuf;

use orion_ai::client::{load_key_dict, AiClientBuilder};
use orion_ai::func::git::{create_git_functions, GitFunctionExecutor};
use orion_ai::provider::AiRequest;
use orion_ai::{AiConfig, FunctionExecutor, FunctionRegistry};
use orion_variate::vars::EnvEvalable;

#[tokio::main]
async fn main() -> orion_ai::AiResult<()> {
    env_logger::init();
    // 1. 配置 DeepSeek API
    let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        println!("✅ 使用  API 进行 Git 操作");
        AiConfig::example().env_eval(&dict)
    } else {
        println!("❌ 错误: 需要配置 k API 密钥");
        println!("请设置 sec_deepseek_api_key 配置!");
        return Ok(());
    };

    // 2. 创建客户端
    let client = AiClientBuilder::new(config)
        .with_role(PathBuf::from("./examples/ai-roles.yml"))
        .build()?;

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

    println!("\n🔧 注册的 Git 函数:");
    for func in registry.get_functions() {
        println!("   - {}: {}", func.name, func.description);
    }

    // 6. 场景1: 检查Git状态
    println!("\n=== 📊 场景1: 检查Git状态 ===");
    let status_request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt(
            "你是一个Git助手。当用户要求检查Git状态时，你必须调用git_status函数。".to_string(),
        )
        .user_prompt("请检查当前Git仓库的状态，看看有哪些文件被修改了".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("📤 发送Git状态检查请求...");
    let status_response = client
        .send_request_with_functions(status_request, &registry)
        .await?;

    match &status_response.tool_calls {
        Some(function_calls) => {
            println!("✅ AI 请求执行Git状态检查");
            for function_call in function_calls {
                println!("   - 调用函数: {}", function_call.function.name);
            }

            println!("\n⚙️ 执行Git状态检查...");
            let status_result = client
                .handle_function_calls(&status_response, &registry)
                .await?;
            println!("📋 Git状态结果:\n{}", status_result);
        }
        None => {
            println!("❌ AI 没有调用Git函数，返回文本响应:");
            println!("📝 {}", status_response.content);
        }
    }

    // 7. 场景2: 添加修改的文件
    println!("\n=== ➕ 场景2: 添加修改的文件 ===");
    let add_request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt("你是一个Git助手。当用户要求添加文件时，你必须调用git_add函数。".to_string())
        .user_prompt("请将所有修改的文件添加到Git暂存区".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("📤 发送添加文件请求...");
    let add_response = client
        .send_request_with_functions(add_request, &registry)
        .await?;

    match &add_response.tool_calls {
        Some(function_calls) => {
            println!("✅ AI 请求添加文件");
            for function_call in function_calls {
                println!("   - 调用函数: {}", function_call.function.name);
            }

            println!("\n⚙️ 执行添加文件操作...");
            let add_result = client
                .handle_function_calls(&add_response, &registry)
                .await?;
            println!("📁 添加文件结果:\n{}", add_result);
        }
        None => {
            println!("❌ AI 没有调用Git函数，返回文本响应:");
            println!("📝 {}", add_response.content);
        }
    }

    // 8. 场景3: 创建提交
    println!("\n=== 💾 场景3: 创建提交 ===");
    let commit_request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt(
            "你是一个Git助手。当用户要求创建提交时，你必须调用git_commit函数。".to_string(),
        )
        .user_prompt("请创建一个提交，提交消息为'feat: 添加function calling功能支持'".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("📤 发送提交请求...");
    let commit_response = client
        .send_request_with_functions(commit_request, &registry)
        .await?;

    match &commit_response.tool_calls {
        Some(function_calls) => {
            println!("✅ AI 请求创建提交");
            for function_call in function_calls {
                println!("   - 调用函数: {}", function_call.function.name);
            }

            println!("\n⚙️ 执行提交操作...");
            let commit_result = client
                .handle_function_calls(&commit_response, &registry)
                .await?;
            println!("💾 提交结果:\n{}", commit_result);
        }
        None => {
            println!("❌ AI 没有调用Git函数，返回文本响应:");
            println!("📝 {}", commit_response.content);
        }
    }

    // 9. 场景4: 推送到远程仓库
    println!("\n=== 🚀 场景4: 推送到远程仓库 ===");
    let push_request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt(
            "你是一个Git助手。当用户要求推送代码时，你必须调用git_push函数。".to_string(),
        )
        .user_prompt("请将提交推送到远程仓库".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("📤 发送推送请求...");
    let push_response = client
        .send_request_with_functions(push_request, &registry)
        .await?;

    match &push_response.tool_calls {
        Some(function_calls) => {
            println!("✅ AI 请求推送代码");
            for function_call in function_calls {
                println!("   - 调用函数: {}", function_call.function.name);
            }

            println!("\n⚙️ 执行推送操作...");
            let push_result = client
                .handle_function_calls(&push_response, &registry)
                .await?;
            println!("🚀 推送结果:\n{}", push_result);
        }
        None => {
            println!("❌ AI 没有调用Git函数，返回文本响应:");
            println!("📝 {}", push_response.content);
        }
    }

    // 10. 场景5: 完整Git工作流
    println!("\n=== 🔄 场景5: 完整Git工作流 ===");
    let workflow_request = AiRequest::builder()
        .model("deepseek-chat")
        .system_prompt(
            "你是一个Git助手。当用户要求执行完整的Git工作流时，你必须按顺序调用相应的函数：git_status -> git_add -> git_commit -> git_push".to_string(),
        )
        .user_prompt(
            "请帮我执行完整的Git工作流：检查状态、添加所有修改的文件、创建提交（消息为'完整workflow测试'）、然后推送到远程仓库".to_string()
        )
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("📤 发送完整工作流请求...");
    let workflow_response = client
        .send_request_with_functions(workflow_request, &registry)
        .await?;

    match &workflow_response.tool_calls {
        Some(function_calls) => {
            println!("✅ AI 请求执行完整工作流");
            println!("   计划执行 {} 个函数:", function_calls.len());
            for (i, function_call) in function_calls.iter().enumerate() {
                println!("   {}. {}", i + 1, function_call.function.name);
            }

            println!("\n⚙️ 执行完整工作流...");
            let workflow_result = client
                .handle_function_calls(&workflow_response, &registry)
                .await?;
            println!("🎯 完整工作流结果:\n{}", workflow_result);
        }
        None => {
            println!("❌ AI 没有调用Git函数，返回文本响应:");
            println!("📝 {}", workflow_response.content);
        }
    }

    // 11. 总结
    println!("\n🎉 Git 工作流示例完成！");
    Ok(())
}
