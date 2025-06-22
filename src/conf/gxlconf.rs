use derive_getters::Getters;
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Getters)]
pub struct GxlConf {
    report_enable: bool,
    report_svr: ReportCenterConf,
}
impl GxlConf {
    pub fn new(conf: ReportCenterConf, enable: bool) -> Self {
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
    pub fn new<S: Into<String>>(domain: S, port: u16) -> Self {
        Self {
            domain: domain.into(),
            port,
        }
    }

    // 定义一个名为local的函数，返回Self类型
    pub fn local() -> Self {
        // 调用Self的new方法，传入参数
        Self::new(
            // 传入IP地址
            "127.0.0.1",
            // 传入端口号
            8066,
        )
    }
}
