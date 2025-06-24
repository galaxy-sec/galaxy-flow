use crate::{task_report::task_rc_config::TASK_REPORT_CENTER, ExecReason};
use colored::Colorize;
use orion_error::StructError;
use serde::Serialize;
use std::fmt::Debug;

// 发送http请求
pub async fn send_http_request<T: Serialize + Debug>(payload: T, url: &String) {
    // 检查报告中心是否启用，并在作用域结束时释放读锁
    let should_send = {
        let task_config = TASK_REPORT_CENTER.get();
        if let Some(task_config_lock) = task_config {
            let task_config = task_config_lock.read().await;
            task_config.report_enable
        } else {
            false // 如果没有配置，默认关闭
        }
    };

    if !should_send {
        return; // 如果报告中心未启用，则直接返回不再发送http请求
    }

    let response = reqwest::Client::new()
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| {
            // Convert reqwest::Error to your ExecReason type, then into StructError
            let exec_reason = ExecReason::NetWork(format!("HTTP request failed: {}", e));
            StructError::from(exec_reason)
        });

    match response {
        Ok(resp) => {
            if resp.status().is_success() {
                debug!("HTTP request to {} succeeded", url);
            } else {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                println!(
                    "{}",
                    format!(
                        "HTTP request to {} failed with status {}: {}",
                        url, status, text
                    )
                    .yellow()
                    .bold()
                );
                // 在这里获取写锁，此时读锁已经释放
                if let Some(task_config_lock) = TASK_REPORT_CENTER.get() {
                    let mut task_config = task_config_lock.write().await;
                    task_config.report_enable = false; // Disable reporting if the request fails
                }
            }
        }
        Err(e) => {
            println!(
                "{}",
                format!("HTTP request to {} failed: {}", url, e)
                    .yellow()
                    .bold()
            );
            // 在这里获取写锁，此时读锁已经释放
            if let Some(task_config_lock) = TASK_REPORT_CENTER.get() {
                let mut task_config = task_config_lock.write().await;
                task_config.report_enable = false; // Disable reporting if the request fails
            }
        }
    }
}
