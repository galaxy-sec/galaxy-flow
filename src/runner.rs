use crate::{
    GxLoader,
    ability::prelude::TaskValue,
    cmd::GxlCmd,
    err::{RunReason, RunResult},
    execution::VarSpace,
    util::redirect::ReadSignal,
};
use orion_error::{ErrorConv, ErrorWith, UvsConfFrom};
use std::{path::Path, sync::mpsc::Sender};

/// Galaxy Flow 运行器
///
/// GxlRunner负责执行GxlCmd命令，加载配置文件并运行指定的流程。
///
/// Galaxy Flow Runner
///
/// GxlRunner is responsible for executing GxlCmd commands, loading configuration files, and running specified flows.
pub struct GxlRunner {}
impl GxlRunner {
    /// 执行Galaxy Flow命令
    ///
    /// 此方法执行以下步骤：
    /// 1. 验证命令参数
    /// 2. 加载配置文件
    /// 3. 解析流程名称
    /// 4. 执行指定的流程
    ///
    /// Execute Galaxy Flow command
    ///
    /// This method performs the following steps:
    /// 1. Validate command parameters
    /// 2. Load configuration file
    /// 3. Parse flow names
    /// 4. Execute specified flows
    pub async fn run(
        cmd: GxlCmd,
        vars: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> RunResult<TaskValue> {
        let loader = GxLoader::new();
        if let Some(ref conf) = cmd.conf {
            // 检查配置文件是否存在 / Check if configuration file exists
            if !Path::new(conf.as_str()).exists() {
                return Err(RunReason::from_conf("gflow conf not exists".to_string()).into())
                    .with(("conf", conf.clone()));
            }

            let spc = loader
                .parse_file(conf.as_str(), cmd.mod_update, &vars)
                .await?
                .assemble()
                .err_conv()?;

            if cmd.flows.is_empty() {
                spc.show().err_conv()?;
            } else {
                // 解析环境列表 / Parse environment list
                return spc.exec(cmd, vars, sender).await;
            }
        }
        Err(RunReason::from_conf("gflow exec fail!".to_string()).into())
    }
}
