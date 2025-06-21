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
}
impl ReportCenterConf {
    fn new<S: Into<String>>(domain: S, port: u16) -> Self {
        Self {
            domain: domain.into(),
            port,
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
