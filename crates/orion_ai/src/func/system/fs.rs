use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiErrReason, AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult,
};

use super::{execute_command_with_timeout, parse_function_arguments, validate_and_normalize_path};

// 文件系统函数执行器
pub struct FileSystemExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for FileSystemExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "fs-ls" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");

                let safe_path = validate_and_normalize_path(path)?;

                match execute_command_with_timeout("ls", &["-la", &safe_path], 10).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "fs-ls".to_string(),
                            result: json!({
                                "path": safe_path,
                                "output": result,
                                "success": output.status.success()
                            }),
                            error: if output.status.success() {
                                None
                            } else {
                                Some(String::from_utf8_lossy(&output.stderr).to_string())
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "fs-ls".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to list directory: {}", e)),
                    }),
                }
            }

            "fs-pwd" => match execute_command_with_timeout("pwd", &[], 5).await {
                Ok(output) => {
                    let result = String::from_utf8_lossy(&output.stdout).to_string();
                    Ok(FunctionResult {
                        name: "fs-pwd".to_string(),
                        result: json!({
                            "current_directory": result.trim(),
                            "success": output.status.success()
                        }),
                        error: if output.status.success() {
                            None
                        } else {
                            Some(String::from_utf8_lossy(&output.stderr).to_string())
                        },
                    })
                }
                Err(e) => Ok(FunctionResult {
                    name: "fs-pwd".to_string(),
                    result: serde_json::Value::Null,
                    error: Some(format!("Failed to get current directory: {}", e)),
                }),
            },

            "fs-cat" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str()).ok_or_else(|| {
                    AiErrReason::from_logic("path parameter is required".to_string()).to_err()
                })?;

                let safe_path = validate_and_normalize_path(path)?;

                match execute_command_with_timeout("cat", &[&safe_path], 15).await {
                    Ok(output) => {
                        let content = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "fs-cat".to_string(),
                            result: json!({
                                "path": safe_path,
                                "content": content,
                                "success": output.status.success()
                            }),
                            error: if output.status.success() {
                                None
                            } else {
                                Some(String::from_utf8_lossy(&output.stderr).to_string())
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "fs-cat".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to read file: {}", e)),
                    }),
                }
            }

            "fs-find" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str()).unwrap_or(".");
                let pattern = args.get("pattern").and_then(|v| v.as_str()).unwrap_or("*");

                let safe_path = validate_and_normalize_path(path)?;

                match execute_command_with_timeout("find", &[&safe_path, "-name", pattern], 30)
                    .await
                {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let files: Vec<String> = result.lines().map(|s| s.to_string()).collect();
                        Ok(FunctionResult {
                            name: "fs-find".to_string(),
                            result: json!({
                                "path": safe_path,
                                "pattern": pattern,
                                "files": files,
                                "success": output.status.success()
                            }),
                            error: if output.status.success() {
                                None
                            } else {
                                Some(String::from_utf8_lossy(&output.stderr).to_string())
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "fs-find".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to find files: {}", e)),
                    }),
                }
            }

            _ => Err(AiErrReason::from_logic("Unknown filesystem function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "fs-ls".to_string(),
            "fs-pwd".to_string(),
            "fs-cat".to_string(),
            "fs-find".to_string(),
        ]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_fs_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_fs_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "fs-ls".to_string(),
            description: "列出目录内容".to_string(),
            parameters: vec![FunctionParameter {
                name: "path".to_string(),
                description: "要列出的目录路径，默认为当前目录".to_string(),
                r#type: "string".to_string(),
                required: false,
            }],
        },
        FunctionDefinition {
            name: "fs-pwd".to_string(),
            description: "显示当前工作目录".to_string(),
            parameters: vec![],
        },
        FunctionDefinition {
            name: "fs-cat".to_string(),
            description: "显示文件内容".to_string(),
            parameters: vec![FunctionParameter {
                name: "path".to_string(),
                description: "要读取的文件路径".to_string(),
                r#type: "string".to_string(),
                required: true,
            }],
        },
        FunctionDefinition {
            name: "fs-find".to_string(),
            description: "查找文件".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "path".to_string(),
                    description: "搜索的起始路径，默认为当前目录".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "pattern".to_string(),
                    description: "文件名模式，支持通配符，默认为 *".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_filesystem_executor() {
        let executor = FileSystemExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert!(functions.contains(&"fs-ls".to_string()));
        assert!(functions.contains(&"fs-pwd".to_string()));
        assert!(functions.contains(&"fs-cat".to_string()));
        assert!(functions.contains(&"fs-find".to_string()));
    }

    #[tokio::test]
    async fn test_create_fs_functions() {
        let functions = create_fs_functions();
        assert_eq!(functions.len(), 4);

        // 验证函数定义
        let ls_func = functions.iter().find(|f| f.name == "fs-ls").unwrap();
        assert_eq!(ls_func.description, "列出目录内容");
        assert_eq!(ls_func.parameters.len(), 1);
        assert_eq!(ls_func.parameters[0].name, "path");
        assert!(!ls_func.parameters[0].required);
    }

    #[test]
    fn test_parse_function_arguments() {
        // 测试空参数
        let result = parse_function_arguments("{}");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);

        // 测试有效参数
        let args = r#"{"path": "/tmp", "pattern": "*.txt"}"#;
        let result = parse_function_arguments(args);
        assert!(result.is_ok());
        let parsed = result.unwrap();
        assert_eq!(parsed.get("path").unwrap().as_str().unwrap(), "/tmp");
        assert_eq!(parsed.get("pattern").unwrap().as_str().unwrap(), "*.txt");
    }
}
