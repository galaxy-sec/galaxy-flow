use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiErrReason, AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult,
};

use super::{execute_command_with_timeout, parse_function_arguments};

// 系统信息函数执行器
pub struct SystemInfoExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for SystemInfoExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "sys-uname" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let detailed = args
                    .get("detailed")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                let command_args = if detailed { vec!["-a"] } else { vec!["-s"] };

                match execute_command_with_timeout("uname", &command_args, 10).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        Ok(FunctionResult {
                            name: "sys-uname".to_string(),
                            result: json!({
                                "output": result.trim(),
                                "detailed": detailed,
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
                        name: "sys-uname".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get system info: {}", e)),
                    }),
                }
            }

            "sys-ps" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let process_name = args.get("process_name").and_then(|v| v.as_str());
                let user = args.get("user").and_then(|v| v.as_str());

                let mut command_args = vec!["aux"];

                // 添加过滤条件
                if let Some(name) = process_name {
                    command_args.push("|");
                    command_args.push("grep");
                    command_args.push(name);
                }

                if let Some(username) = user {
                    command_args.push("|");
                    command_args.push("grep");
                    command_args.push(username);
                }

                match execute_command_with_timeout("ps", &command_args, 15).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        Ok(FunctionResult {
                            name: "sys-ps".to_string(),
                            result: json!({
                                "processes": lines,
                                "process_filter": process_name,
                                "user_filter": user,
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
                        name: "sys-ps".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get process info: {}", e)),
                    }),
                }
            }

            "sys-df" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let path = args.get("path").and_then(|v| v.as_str());
                let human_readable = args
                    .get("human_readable")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);

                let mut command_args = if human_readable { vec!["-h"] } else { vec![] };

                if let Some(p) = path {
                    command_args.push(p);
                } else {
                    command_args.push(".");
                }

                match execute_command_with_timeout("df", &command_args, 10).await {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let lines: Vec<String> = result.lines().map(|s| s.to_string()).collect();

                        Ok(FunctionResult {
                            name: "sys-df".to_string(),
                            result: json!({
                                "disk_usage": lines,
                                "path": path.unwrap_or("."),
                                "human_readable": human_readable,
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
                        name: "sys-df".to_string(),
                        result: serde_json::Value::Null,
                        error: Some(format!("Failed to get disk usage: {}", e)),
                    }),
                }
            }

            _ => Err(AiErrReason::from_logic("Unknown system info function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec![
            "sys-uname".to_string(),
            "sys-ps".to_string(),
            "sys-df".to_string(),
        ]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_sys_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_sys_functions() -> Vec<FunctionDefinition> {
    vec![
        FunctionDefinition {
            name: "sys-uname".to_string(),
            description: "显示系统信息".to_string(),
            parameters: vec![FunctionParameter {
                name: "detailed".to_string(),
                description: "是否显示详细信息，默认为false".to_string(),
                r#type: "boolean".to_string(),
                required: false,
            }],
        },
        FunctionDefinition {
            name: "sys-ps".to_string(),
            description: "显示进程信息".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "process_name".to_string(),
                    description: "按进程名过滤，可选".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "user".to_string(),
                    description: "按用户名过滤，可选".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
            ],
        },
        FunctionDefinition {
            name: "sys-df".to_string(),
            description: "显示磁盘使用情况".to_string(),
            parameters: vec![
                FunctionParameter {
                    name: "path".to_string(),
                    description: "要检查的路径，默认为当前目录".to_string(),
                    r#type: "string".to_string(),
                    required: false,
                },
                FunctionParameter {
                    name: "human_readable".to_string(),
                    description: "是否使用人类可读格式，默认为true".to_string(),
                    r#type: "boolean".to_string(),
                    required: false,
                },
            ],
        },
    ]
}

// 解析函数参数的辅助函数 - 使用公共版本
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_system_info_executor() {
        let executor = SystemInfoExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert!(functions.contains(&"sys-uname".to_string()));
        assert!(functions.contains(&"sys-ps".to_string()));
        assert!(functions.contains(&"sys-df".to_string()));
    }

    #[tokio::test]
    async fn test_create_sys_functions() {
        let functions = create_sys_functions();
        assert_eq!(functions.len(), 3);

        // 验证 uname 函数定义
        let uname_func = functions.iter().find(|f| f.name == "sys-uname").unwrap();
        assert_eq!(uname_func.description, "显示系统信息");
        assert_eq!(uname_func.parameters.len(), 1);
        assert_eq!(uname_func.parameters[0].name, "detailed");
        assert!(!uname_func.parameters[0].required);

        // 验证 ps 函数定义
        let ps_func = functions.iter().find(|f| f.name == "sys-ps").unwrap();
        assert_eq!(ps_func.description, "显示进程信息");
        assert_eq!(ps_func.parameters.len(), 2);
        assert_eq!(ps_func.parameters[0].name, "process_name");
        assert!(!ps_func.parameters[0].required);
        assert_eq!(ps_func.parameters[1].name, "user");
        assert!(!ps_func.parameters[1].required);

        // 验证 df 函数定义
        let df_func = functions.iter().find(|f| f.name == "sys-df").unwrap();
        assert_eq!(df_func.description, "显示磁盘使用情况");
        assert_eq!(df_func.parameters.len(), 2);
        assert_eq!(df_func.parameters[0].name, "path");
        assert!(!df_func.parameters[0].required);
        assert_eq!(df_func.parameters[1].name, "human_readable");
        assert!(!df_func.parameters[1].required);
    }
}
