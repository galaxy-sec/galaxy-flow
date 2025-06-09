use orion_error::StructError;

use crate::{task_result::TaskResult, ExecReason, ExecResult};

pub async fn send_http_request(payload: TaskResult) -> ExecResult<reqwest::Response> {
    reqwest::Client::new()
        .post("http://localhost:8082/api/subtask_result")
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            // Convert reqwest::Error to your ExecReason type, then into StructError
            let exec_reason = ExecReason::NetWork(format!("HTTP request failed: {}", e));
            StructError::from(exec_reason)
        })
}
