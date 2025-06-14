use crate::{
    components::gxl_spc::GxlSpace,
    err::{RunError, RunReason, RunResult},
    execution::VarSpace,
    infra::DfxArgsGetter,
    GxLoader,
};
use clap::ArgAction;
use orion_error::{ErrorConv, ErrorWith, StructError, UvsConfFrom, UvsReason};
use std::path::Path;

pub struct GxlRunner {}
impl GxlRunner {
    #[allow(clippy::result_large_err)]
    pub async fn run(cmd: GxlCmd, vars: VarSpace) -> RunResult<()> {
        let mut loader = GxLoader::new();
        if let Some(conf) = cmd.conf {
            if !Path::new(conf.as_str()).exists() {
                return Err(StructError::from_conf("conf not exists".to_string()))
                    .with(("conf", conf));
            }
            let expect = ShellOption {
                outer_print: cmd.cmd_print,
                ..Default::default()
            };
            let spc = GxlSpace::try_from(loader.parse_file(conf.as_str(), false, expect, &vars)?)
                .err_conv()?;
            if cmd.flow.is_empty() {
                spc.show().err_conv()?;
                return Ok(());
            } else {
                let envs: Vec<String> = cmd.env.split(',').map(String::from).collect();
                let flws: Vec<String> = if cmd.flow.len() == 1 {
                    cmd.flow[0].split(',').map(String::from).collect()
                } else {
                    cmd.flow.clone()
                    //cmd.flow.iter().collect()
                };
                spc.exec(envs, flws, cmd.cmd_print, vars).await?;
                println!("\ngod job!");
            }
            Ok(())
        } else {
            Err(RunError::from(RunReason::from(UvsReason::core_conf(
                "conf is empty".to_string(),
            ))))
        }
    }
}
use crate::model::expect::ShellOption;
use clap::Parser;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(version, about, long_about = None)]
pub struct GxlCmd {
    /// env name ; eg: -e dev
    #[arg(short = 'e', long = "env", default_value = "default")]
    pub env: String,
    /// flow name ; eg: conf,test,package
    pub flow: Vec<String>,
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// conf file ;  default is  work(./_rg/work.gxl) adm (./_rg/adm.gxl)
    #[arg(short = 'f', long = "conf")]
    pub conf: Option<String>,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
    #[arg(short= 'q', long = "quiet" ,action = ArgAction::SetFalse , default_value = "true")]
    pub cmd_print: bool,
    #[arg( allow_hyphen_values = true,  // 关键设置：允许值以 - 开头
        last = true,                 // 关键设置：捕获所有剩余参数
        value_name = "cmd_args",
        default_value = ""
    )]
    pub cmd_args: String,
}
impl DfxArgsGetter for GxlCmd {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
