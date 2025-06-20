use std::{env, fmt::Debug};

use orion_error::StructError;
use serde::Serialize;

use crate::{task_report::task_rc_config::TASK_REPORT_CENTER, ExecReason};

pub fn get_task_notice_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_result_center") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        return Some(format!(
            "http://{}:{}{}",
            report_svr.domain, report_svr.port, report_svr.task_notice_center
        ));
    }
    None
}

pub fn get_task_report_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_report_center") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        return Some(format!(
            "http://{}:{}{}",
            report_svr.domain, report_svr.port, report_svr.task_report_center
        ));
    }
    None
}

pub fn get_main_task_create_url() -> Option<String> {
    if let Ok(url) = env::var("main_task_create_center") {
        return Some(url);
    }
    let task_config = TASK_REPORT_CENTER.get();
    if let Some(task_config) = task_config {
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        return Some(format!(
            "http://{}:{}{}",
            report_svr.domain, report_svr.port, report_svr.main_task_create_center
        ));
    }
    None
}

// 发送http请求
pub async fn send_http_request<T: Serialize + Debug>(payload: T, url: &String) {
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
                    "HTTP request to {} failed with status {}: {}",
                    url, status, text
                );
            }
        }
        Err(e) => {
            println!("HTTP request to {} failed: {}", url, e);
        }
    }
}
