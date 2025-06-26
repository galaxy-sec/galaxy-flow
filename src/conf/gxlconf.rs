use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

use crate::task_report::task_rc_config::{ReportCenterConf, TaskCenterAPI};

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters)]
pub struct GxlConf {
    task_rc_config: TaskCenterAPI,
}
impl GxlConf {
    pub fn new(conf: ReportCenterConf, enable: bool) -> Self {
        Self {
            task_rc_config: TaskCenterAPI {
                report_enable: enable,
                report_svr: conf,
            },
        }
    }
}
