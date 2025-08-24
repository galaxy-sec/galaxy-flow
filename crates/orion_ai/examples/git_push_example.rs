use orion_ai::client::load_key_dict;
use orion_ai::provider::AiRequest;
use orion_ai::{
    AiClient, AiConfig, FunctionDefinition, FunctionParameter, FunctionRegistry, FunctionResult,
};
use orion_error::ToStructError;
use orion_error::UvsLogicFrom;
use orion_variate::vars::EnvEvalable;
use serde_json::json;

// Git 函数执行器
pub struct GitFunctionExecutor;

#[async_trait::async_trait]
impl orion_ai::FunctionExecutor for GitFunctionExecutor {
    async fn execute(
        &self,
        function_call: &orion_ai::FunctionCall,
    ) -> orion_ai::AiResult<FunctionResult> {
        match function_call.name.as_str() {
            "git_status" => {
                let path = function_call
                    .arguments
                    .get("path")
                    .and_then(|v| v.as_str())
                    .unwrap_or(".");

                match tokio::process::Command::new("git")
                    .args(["status", "--porcelain"])
                    .current_dir(path)
                    .output()
                    .await
                {
                    Ok(output) => {
                        let status = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "git_status".to_string(),
                            result: json!({
                                "status": status,
                                "has_changes": !status.trim().is_empty()
                            }),
                            error: None,
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git_status".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get git status: {}", e)),
                    }),
                }
            }

            "git_add" => {
                let files = function_call
                    .arguments
                    .get("files")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| {
                        orion_ai::AiErrReason::from_logic(
                            "TODO: files parameter required".to_string(),
                        )
                        .to_err()
                    })?;

                let file_list: Vec<String> = files
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect();

                match tokio::process::Command::new("git")
                    .args(["add"])
                    .args(file_list)
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git_add".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": "Files added successfully"
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git_add".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git_add".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to add files: {}", e)),
                    }),
                }
            }

            "git_commit" => {
                let message = function_call
                    .arguments
                    .get("message")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| {
                        orion_ai::AiErrReason::from_logic(
                            "TODO: message parameter required".to_string(),
                        )
                        .to_err()
                    })?;

                match tokio::process::Command::new("git")
                    .args(["commit", "-m", message])
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git_commit".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": "Commit created successfully"
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git_commit".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git_commit".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to create commit: {}", e)),
                    }),
                }
            }

            "git_push" => {
                let remote = function_call
                    .arguments
                    .get("remote")
                    .and_then(|v| v.as_str())
                    .unwrap_or("origin");
                let branch = function_call
                    .arguments
                    .get("branch")
                    .and_then(|v| v.as_str())
                    .unwrap_or("HEAD");

                match tokio::process::Command::new("git")
                    .args(["push", remote, branch])
                    .output()
                    .await
                {
                    Ok(output) => {
                        if output.status.success() {
                            Ok(FunctionResult {
                                name: "git_push".to_string(),
                                result: json!({
                                    "success": true,
                                    "message": format!("Pushed to {}/{}", remote, branch)
                                }),
                                error: None,
                            })
                        } else {
                            let error_msg = String::from_utf8_lossy(&output.stderr);
                            Ok(FunctionResult {
                                name: "git_push".to_string(),
                                result: serde_json::Value::Null,
                                error: Some(error_msg.to_string()),
                            })
                        }
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "git_push".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to push: {}", e)),
                    }),
                }
            }

            _ => Err(
                orion_ai::AiErrReason::from_logic("TODO: unknown function".to_string()).to_err(),
            ),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "git_status".to_string(),
            "git_add".to_string(),
            "git_commit".to_string(),
            "git_push".to_string(),
        ]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_git_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

// 创建 Git 函数定义
pub fn create_git_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "git_status".to_string(),
            description: "获取Git仓库状态".to_string(),
            parameters: vec![FunctionParameter {
                name: "path".to_string(),
                description: "仓库路径，默认为当前目录".to_string(),
                r#type: "string".to_string(),
                required: false,
            }],
        },
        FunctionDefinition {
            name: "git_add".to_string(),
            description: "添加文件到Git暂存区".to_string(),
            parameters: vec![FunctionParameter {
                name: "files".to_string(),
                description: "要添加的文件列表，支持通配符".to_string(),
                r#type: "array".to_string(),
                required: true,
            }],
        },
        FunctionDefinition {
            name: "git_commit".to_string(),
            description: "创建Git提交".to_string(),
            parameters: vec![FunctionParameter {
                name: "message".to_string(),
                description: "提交消息".to_string(),
                r#type: "string".to_string(),
                required: true,
            }],
        },
        FunctionDefinition {
            name: "git_push".to_string(),
            description: "推送提交到远程仓库".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "remote".to_string(),
                    description: "远程仓库名称，默认为origin".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "branch".to_string(),
                    description: "分支名称，默认为当前分支".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

#[tokio::main]
async fn main() -> orion_ai::AiResult<()> {
    let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        AiConfig::example().env_eval(&dict)
    } else {
        return Ok(());
    };
    // 1. 创建客户端
    let client = AiClient::new(config, None)?;

    // 2. 创建函数注册表
    let mut registry = FunctionRegistry::new();

    // 3. 注册Git函数
    let git_functions = create_git_functions();
    for function in git_functions {
        registry.register_function(function)?;
    }

    // 4. 注册Git执行器
    registry.register_executor("git".to_string(), std::sync::Arc::new(GitFunctionExecutor))?;

    // 5. 发送Git相关的AI请求
    let request = AiRequest::builder()
        .model("gpt-4o")
        .system_prompt("你是一个Git助手，可以帮助用户进行版本控制操作。当用户询问Git相关问题时，你可以调用相应的函数来帮助他们。".to_string())
        .user_prompt("请帮我检查当前Git状态，然后添加所有修改的文件，创建一个提交，消息为'fix: update documentation'，最后推送到远程仓库。".to_string())
        .functions(create_git_functions())
        .enable_function_calling(true)
        .build();

    println!("发送AI请求...");
    let response = client
        .send_request_with_functions(request, &registry)
        .await?;

    // 6. 处理函数调用
    if let Some(function_calls) = &response.function_calls {
        println!("AI请求执行以下函数调用:");
        for function_call in function_calls {
            println!("- 调用函数: {}", function_call.name);
            println!("  参数: {:?}", function_call.arguments);
        }

        println!("\n执行函数调用...");
        let final_result = client.handle_function_calls(&response, &registry).await?;
        println!("\n最终结果:");
        println!("{}", final_result);
    } else {
        println!("AI响应: {}", response.content);
    }

    Ok(())
}
