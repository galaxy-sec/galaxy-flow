use orion_error::{ToStructError, UvsLogicFrom};
use serde_json::json;

use crate::{
    AiErrReason, AiResult, FunctionCall, FunctionDefinition, FunctionExecutor, FunctionParameter,
    FunctionResult,
};

use super::{execute_command_with_timeout, parse_function_arguments};

// 网络工具函数执行器
pub struct NetworkExecutor;

#[async_trait::async_trait]
impl FunctionExecutor for NetworkExecutor {
    async fn execute(&self, function_call: &FunctionCall) -> AiResult<FunctionResult> {
        match function_call.function.name.as_str() {
            "net-ping" => {
                let args = parse_function_arguments(&function_call.function.arguments)?;
                let host = args.get("host").and_then(|v| v.as_str()).ok_or_else(|| {
                    AiErrReason::from_logic("host parameter is required".to_string()).to_err()
                })?;
                let count = args.get("count").and_then(|v| v.as_u64()).unwrap_or(4);
                let timeout = args.get("timeout").and_then(|v| v.as_u64()).unwrap_or(10);

                // 验证主机名安全性（防止注入攻击）
                if !is_valid_host(host) {
                    return Err(AiErrReason::from_logic(format!("Invalid host: {}", host)).to_err());
                }

                // 限制ping数量以防止滥用
                let count = if count > 10 { 10 } else { count as i32 };
                let timeout_seconds = if timeout > 30 { 30 } else { timeout as u64 };

                match execute_command_with_timeout(
                    "ping",
                    &[
                        "-c",
                        &count.to_string(),
                        "-W",
                        &timeout_seconds.to_string(),
                        host,
                    ],
                    timeout_seconds + 5, // 额外缓冲时间
                )
                .await
                {
                    Ok(output) => {
                        let result = String::from_utf8_lossy(&output.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

                        // 解析ping结果
                        let success = output.status.success();
                        let mut parsed_result = json!({
                            "host": host,
                            "count": count,
                            "timeout_seconds": timeout_seconds,
                            "success": success,
                            "raw_output": result
                        });

                        // 尝试提取统计信息
                        if success {
                            if let Some(stats) = parse_ping_stats(&result) {
                                parsed_result["stats"] = stats;
                            }
                        }

                        Ok(FunctionResult {
                            name: "net-ping".to_string(),
                            result: parsed_result,
                            error: if !success && !stderr.is_empty() {
                                Some(stderr.trim().to_string())
                            } else if !success {
                                Some("Ping failed".to_string())
                            } else {
                                None
                            },
                        })
                    }
                    Err(e) => Ok(FunctionResult {
                        name: "net-ping".to_string(),
                        result: json!({
                            "host": host,
                            "success": false
                        }),
                        error: Some(format!("Failed to execute ping: {}", e)),
                    }),
                }
            }

            _ => Err(AiErrReason::from_logic("Unknown network function".to_string()).to_err()),
        }
    }

    fn supported_functions(&self) -> Vec<String> {
        vec!["net-ping".to_string()]
    }

    fn get_function_schema(&self, function_name: &str) -> Option<FunctionDefinition> {
        create_net_functions()
            .into_iter()
            .find(|f| f.name == function_name)
    }
}

pub fn create_net_functions() -> Vec<FunctionDefinition> {
    vec![FunctionDefinition {
        name: "net-ping".to_string(),
        description: "测试网络连通性".to_string(),
        parameters: vec![
            FunctionParameter {
                name: "host".to_string(),
                description: "要ping的主机名或IP地址".to_string(),
                r#type: "string".to_string(),
                required: true,
            },
            FunctionParameter {
                name: "count".to_string(),
                description: "ping包数量，默认为4，最大为10".to_string(),
                r#type: "number".to_string(),
                required: false,
            },
            FunctionParameter {
                name: "timeout".to_string(),
                description: "超时时间（秒），默认为10，最大为30".to_string(),
                r#type: "number".to_string(),
                required: false,
            },
        ],
    }]
}

// 验证主机名安全性
fn is_valid_host(host: &str) -> bool {
    if host.is_empty() || host.len() > 253 {
        return false;
    }

    // 防止命令注入
    if host.contains(';')
        || host.contains('|')
        || host.contains('&')
        || host.contains('>')
        || host.contains('<')
    {
        return false;
    }

    // 允许的主机名格式：域名或IP地址
    let valid_chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789.-";
    host.chars().all(|c| valid_chars.contains(c))
}

// 解析ping统计信息
fn parse_ping_stats(output: &str) -> Option<serde_json::Value> {
    let lines: Vec<&str> = output.lines().collect();

    // 查找统计信息行
    for line in lines.iter().rev() {
        if line.contains("packets transmitted") {
            return Some(json!({
                "summary": line.trim()
            }));
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_executor() {
        let executor = NetworkExecutor;

        // 测试支持的函数列表
        let functions = executor.supported_functions();
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0], "net-ping");
    }

    #[tokio::test]
    async fn test_create_net_functions() {
        let functions = create_net_functions();
        assert_eq!(functions.len(), 1);

        // 验证函数定义
        let ping_func = &functions[0];
        assert_eq!(ping_func.name, "net-ping");
        assert_eq!(ping_func.description, "测试网络连通性");
        assert_eq!(ping_func.parameters.len(), 3);

        // 验证参数
        let host_param = &ping_func.parameters[0];
        assert_eq!(host_param.name, "host");
        assert_eq!(host_param.r#type, "string");
        assert!(host_param.required);

        let count_param = &ping_func.parameters[1];
        assert_eq!(count_param.name, "count");
        assert_eq!(count_param.r#type, "number");
        assert!(!count_param.required);

        let timeout_param = &ping_func.parameters[2];
        assert_eq!(timeout_param.name, "timeout");
        assert_eq!(timeout_param.r#type, "number");
        assert!(!timeout_param.required);
    }

    #[test]
    fn test_is_valid_host() {
        // 测试有效主机名
        assert!(is_valid_host("example.com"));
        assert!(is_valid_host("localhost"));
        assert!(is_valid_host("192.168.1.1"));
        assert!(is_valid_host("sub.domain.com"));
        assert!(is_valid_host("test-host"));

        // 测试无效主机名
        assert!(!is_valid_host(""));
        assert!(!is_valid_host("a".repeat(254).as_str())); // 过长
        assert!(!is_valid_host("example;com")); // 包含分号
        assert!(!is_valid_host("example|com")); // 包含管道
        assert!(!is_valid_host("example&com")); // 包含and
        assert!(!is_valid_host("example>com")); // 包含重定向
        assert!(!is_valid_host("example<com")); // 包含重定向
    }

    #[test]
    fn test_parse_ping_stats() {
        let ping_output = r#"PING example.com (93.184.216.34) 56(84) bytes of data.
64 bytes from 93.184.216.34 (93.184.216.34): icmp_seq=1 ttl=52 time=15.2 ms
64 bytes from 93.184.216.34 (93.184.216.34): icmp_seq=2 ttl=52 time=15.1 ms
64 bytes from 93.184.216.34 (93.184.216.34): icmp_seq=3 ttl=52 time=15.3 ms
64 bytes from 93.184.216.34 (93.184.216.34): icmp_seq=4 ttl=52 time=15.2 ms

--- example.com ping statistics ---
4 packets transmitted, 4 received, 0% packet loss, time 3005ms
rtt min/avg/max/mdev = 15.123/15.234/15.345/0.123 ms"#;

        let stats = parse_ping_stats(ping_output);
        assert!(stats.is_some());
        let stats = stats.unwrap();
        assert!(stats.is_object());
        assert!(stats["summary"].is_string());
        let summary = stats["summary"].as_str().unwrap();
        assert!(summary.contains("4 packets transmitted"));
    }
}
