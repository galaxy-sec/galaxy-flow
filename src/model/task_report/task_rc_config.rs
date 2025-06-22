use once_cell::sync::OnceCell;
use serde::Deserialize;

lazy_static! {
    pub static ref TASK_REPORT_CENTER: OnceCell<TaskCenterAPI> = OnceCell::new();
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
