use crate::{
    components::gxl_spc::GxlSpace,
    err::{RunError, RunReason, RunResult},
    GxLoader,
};
use clap::ArgAction;
use orion_error::{ErrorConv, ErrorWith, StructError, UvsConfFrom, UvsReason};
use std::path::Path;

pub struct GxlRunner {}
impl GxlRunner {
    #[allow(clippy::result_large_err)]
    pub async fn run(cmd: GxlCmd) -> RunResult<()> {
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
            let spc =
                GxlSpace::try_from(loader.parse_file(conf.as_str(), false, expect)?).err_conv()?;
            if cmd.flow.is_empty() {
                spc.show().err_conv()?;
                return Ok(());
            } else {
                let envs: Vec<&str> = cmd.env.split(',').collect();
                let flws: Vec<&str> = if cmd.flow.len() == 1 {
                    cmd.flow[0].split(',').collect()
                } else {
                    cmd.flow.iter().map(|x| x.as_str()).collect()
                };
                spc.exec(envs, flws, cmd.cmd_print).await?;
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
}
