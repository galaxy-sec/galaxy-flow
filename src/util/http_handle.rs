use crate::{
    execution::task::Task,
    task_report::{
        task_notification::{TaskNotice, TaskOutline},
        task_rc_config::{build_task_url, report_enable, set_report_enable, TaskUrlType},
    },
    ExecReason,
};
use colored::Colorize;
use orion_error::StructError;
use serde::Serialize;
use std::fmt::Debug;

// 发送http请求
pub async fn send_http_request<T: Serialize + Debug>(payload: T, url: &String) {
    if !report_enable().await {
        return; // 如果报告中心未启用，则直接返回不再发送http请求
    }

    let response = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            // Convert reqwest::Error to your ExecReason type, then into StructError
            let exec_reason = ExecReason::NetWork(format!("HTTP request failed: {e}"));
            StructError::<ExecReason>::from(exec_reason)
        });

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                info!(
                    "HTTP request to {} succeeded. {}",
                    url,
                    resp.text().await.unwrap_or_default()
                );
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                println!(
                    "{}",
                    format!("HTTP request to {url} failed with status {status}: {text}",)
                        .yellow()
                        .bold()
                );
                set_report_enable(false).await; // Disable reporting if the request fails
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("HTTP request to {url} failed: {e}",)
                    .yellow()
                    .bold()
            );
            set_report_enable(false).await; // Disable reporting if the request fails
        }
    }
}

// 创建并发送任务通知
pub async fn create_and_send_task_notice(
    task: &Task,
    task_notice: &TaskNotice,
) -> Result<TaskNotice, StructError<ExecReason>> {
    let url = build_task_url(TaskUrlType::TaskNotice)
        .await
        .unwrap_or_default();

    let notice = TaskNotice {
        parent_id: task_notice.parent_id.clone(), // 明确初始化
        name: task.name().to_string(),
        description: task.name().to_string(),
        order: task_notice.order, // 明确初始化
    };

    let task_outline = TaskOutline {
        tasks: vec![notice.clone()],
    };

    send_http_request(task_outline, &url).await;
    Ok(notice)
}
