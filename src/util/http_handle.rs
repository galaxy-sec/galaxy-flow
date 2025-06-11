use std::env;

use orion_error::StructError;

use crate::{
    task_result::{TaskResult, TASK_RESULT_CONDIG},
    ExecReason, ExecResult,
};

pub fn get_task_callback_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_result_center_url") {
        return Some(url);
    }
    let task_config = TASK_RESULT_CONDIG.get();
    if let Some(task_config) = task_config {
        if let Some(task_url) = task_config.task_result_center.clone() {
            return Some(task_url.url);
        }
    }
    None
}

// 先校验是否配置了远程任务结果中心
pub async fn send_http_request(payload: TaskResult, url: &String) -> ExecResult<reqwest::Response> {
    reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            // Convert reqwest::Error to your ExecReason type, then into StructError
            let exec_reason = ExecReason::NetWork(format!("HTTP request failed: {}", e));
            StructError::from(exec_reason)
        })
}
