pub mod fs;
pub mod net;
pub mod sys;

// 重新导出主要的结构体和函数
pub use fs::{create_fs_functions, FileSystemExecutor};
pub use net::{create_net_functions, NetworkExecutor};
pub use sys::{create_sys_functions, SystemInfoExecutor};

use crate::{AiErrReason, AiResult};
use orion_error::{ToStructError, UvsLogicFrom};
use std::time::Duration;
use tokio::process::Command;

/// 带超时的命令执行
pub async fn execute_command_with_timeout(
    command: &str,
    args: &[&str],
    timeout_seconds: u64,
) -> AiResult<std::process::Output> {
    tokio::time::timeout(
        Duration::from_secs(timeout_seconds),
        Command::new(command).args(args).output(),
    )
    .await
    .map_err(|_| AiErrReason::from_logic("Command execution timeout".to_string()).to_err())?
    .map_err(|e| AiErrReason::from_logic(format!("Command execution failed: {}", e)).to_err())
}

/// 验证并规范化路径，防止目录遍历攻击
pub fn validate_and_normalize_path(path: &str) -> AiResult<String> {
    use std::path::PathBuf;

    if path.is_empty() {
        return Ok(".".to_string());
    }

    // 防止目录遍历攻击
    if path.contains("..") || path.contains("~") {
        return Err(AiErrReason::from_logic(
            "Path traversal or home directory access not allowed".to_string(),
        )
        .to_err());
    }

    // 防止命令注入
    if path.contains(';') || path.contains('|') || path.contains('&') || path.contains('$') {
        return Err(
            AiErrReason::from_logic("Path contains invalid characters".to_string()).to_err(),
        );
    }

    // 规范化路径
    let path_buf = PathBuf::from(path);
    let normalized = path_buf.to_string_lossy().to_string();

    Ok(normalized)
}

/// 解析函数参数的辅助函数
pub fn parse_function_arguments(
    arguments: &str,
) -> AiResult<serde_json::Map<String, serde_json::Value>> {
    use serde_json::Value;

    if arguments.trim().is_empty() || arguments == "{}" {
        return Ok(serde_json::Map::new());
    }

    let parsed: Value = serde_json::from_str(arguments).map_err(|e| {
        AiErrReason::from_logic(format!("Failed to parse arguments: {}", e)).to_err()
    })?;

    match parsed {
        Value::Object(map) => Ok(map),
        _ => Err(AiErrReason::from_logic("Arguments must be an object".to_string()).to_err()),
    }
}
