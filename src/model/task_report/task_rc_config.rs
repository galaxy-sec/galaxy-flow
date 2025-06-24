use std::env;

use once_cell::sync::OnceCell;
use serde::Deserialize;
use tokio::sync::RwLock;

lazy_static! {
    pub static ref TASK_REPORT_CENTER: OnceCell<RwLock<TaskCenterAPI>> = OnceCell::new();
}

// 任务结果配置
#[derive(Deserialize, Debug)]
pub struct TaskCenterAPI {
    pub report_enable: bool,
    pub report_svr: ReportSVR,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReportSVR {
    pub domain: String,
    pub port: u16,
}

pub async fn get_task_notice_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_result_center") {
        return Some(url);
    }
    let url = {
        let task_config = TASK_REPORT_CENTER.get()?;
        let task_config = task_config.read().await;
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        format!(
            "http://{}:{}/task/create_batch_subtask/",
            report_svr.domain, report_svr.port,
        )
    };
    Some(url)
}

pub async fn get_task_report_center_url() -> Option<String> {
    if let Ok(url) = env::var("task_report_center") {
        return Some(url);
    }
    let url = {
        let task_config = TASK_REPORT_CENTER.get()?;
        let task_config = task_config.read().await;
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        format!(
            "http://{}:{}/task/update_subtask_info/",
            report_svr.domain, report_svr.port
        )
    };
    Some(url)
}

pub async fn get_main_task_create_url() -> Option<String> {
    if let Ok(url) = env::var("main_task_create_center") {
        return Some(url);
    }
    let url = {
        let task_config = TASK_REPORT_CENTER.get()?;
        let task_config = task_config.read().await;
        if !task_config.report_enable {
            return None; // 如果报告中心未启用，则返回None
        }
        let report_svr = task_config.report_svr.clone();
        format!(
            "http://{}:{}/task/create_main_task/",
            report_svr.domain, report_svr.port
        )
    };
    Some(url)
}
