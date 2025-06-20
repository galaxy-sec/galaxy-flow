mod mod_test;
use std::path::PathBuf;

use derive_getters::Getters;
use orion_error::{ErrorConv, UvsSysFrom};
use orion_syspec::{tools::ensure_path, types::Tomlable};
use serde_derive::{Deserialize, Serialize};

use crate::err::{RunError, RunResult};
#[derive(Serialize, Deserialize, Debug, PartialEq, Getters)]
pub struct GxlConf {
    report_enable: bool,
    report_svr: ReportCenterConf,
}
impl GxlConf {
    fn new(conf: ReportCenterConf, enable: bool) -> Self {
        Self {
            report_enable: enable,
            report_svr: conf,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters)]
pub struct ReportCenterConf {
    domain: String,
    port: u16,
    pub task_notice_center: String,
    pub task_report_center: String,
    pub main_task_create_center: String,
}
impl ReportCenterConf {
    fn new<S: Into<String>>(
        domain: S,
        port: u16,
        task_notice_center: S,
        task_report_center: S,
        main_task_create_center: S,
    ) -> Self {
        Self {
            domain: domain.into(),
            port,
            task_notice_center: task_notice_center.into(),
            task_report_center: task_report_center.into(),
            main_task_create_center: main_task_create_center.into(),
        }
    }

    // 定义一个名为local的函数，返回Self类型
    fn local() -> Self {
        // 调用Self的new方法，传入参数
        Self::new(
            // 传入IP地址
            "127.0.0.1",
            // 传入端口号
            8066,
            // 传入创建批处理子任务的路径
            "/task/create_batch_subtask/",
            // 传入更新子任务信息的路径
            "/task/update_subtask_info/",
            // 传入创建主任务的路径
            "/task/create_main_task/",
        )
    }
}
pub fn conf_path() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        let conf_file = galaxy_root.join("conf.toml");
        if conf_file.exists() {
            return Some(conf_file);
        }
    }
    None
}

pub fn conf_init() -> RunResult<()> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        ensure_path(&galaxy_root).err_conv()?;
        let conf_file = galaxy_root.join("conf.toml");
        let conf = GxlConf::new(ReportCenterConf::local(), true);
        conf.save_toml(&conf_file).err_conv()?;
        return Ok(());
    }
    Err(RunError::from_sys("get home dir failed!".to_string()))
}
