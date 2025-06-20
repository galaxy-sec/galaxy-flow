use std::env;

use orion_error::StructError;
use serde::Serialize;

use crate::{report_center::task_rc_config::TASK_REPORT_CENTER, ExecReason, ExecResult};

pub fn get_task_callback_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_result_center") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if let Some(task_url) = task_config.task_callback_center.clone() {
            return Some(task_url.url);
        }
    }
    None
}

pub fn get_task_report_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_report_center") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if let Some(task_url) = task_config.task_reporting_center.clone() {
            return Some(task_url.url);
        }
    }
    None
}

pub fn get_main_task_create_url() -> Option<String> {
    if let Ok(url) = env::var("main_task_create_url") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if let Some(task_url) = task_config.main_task_create_center.clone() {
            return Some(task_url.url);
        }
    }
    None
}

// 发送http请求
pub async fn send_http_request<T: Serialize>(
    payload: T,
    url: &String,
) -> ExecResult<reqwest::Response> {
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
