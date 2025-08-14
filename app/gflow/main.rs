#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::ai::{AiClient, AiClientTrait, AiConfig, AiRole};
use galaxy_flow::conf::load_gxl_config;
use galaxy_flow::const_val::gxl_const;
use galaxy_flow::err::{report_gxl_error, RunResult};
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::model::task_report::task_rc_config::init_redirect_and_parent_task;
use galaxy_flow::runner::{GxlCmd, GxlRunner};
use galaxy_flow::traits::Setter;
use galaxy_flow::util::redirect::{init_redirect_file, stop_redirect};
use orion_error::{ErrorConv, ErrorOwe};
use orion_variate::vars::EnvDict;
use std::env;
use std::fs::read_to_string;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> RunResult<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init().err_conv()?;

    // 检查是否请求版本信息

    let mut cmd = GxlCmd::parse();
    // 加载task配置

    configure_run_logging(cmd.log.clone(), cmd.debug);
    load_gxl_config();
    let redirect = init_redirect_and_parent_task(cmd.flow.concat())
        .await
        .err_conv()?;
    println!("galaxy-flow : {}", env!("CARGO_PKG_VERSION"));
    debug!("galaxy flow running .....");
    if cmd.conf.is_none() {
        let main_conf = "./_gal/work.gxl";
        cmd.conf = Some(main_conf.to_string());
    }
    var_space
        .global_mut()
        .set(gxl_const::CMD_ARG, cmd.cmd_arg.clone());
    var_space
        .global_mut()
        .set(gxl_const::CMD_DRYRUN, cmd.dryrun);
    var_space
        .global_mut()
        .set(gxl_const::CMD_MODUP, cmd.mod_update);
    match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
        Err(e) => {
            report_gxl_error(e);
            if cmd.ai {
                let output = init_redirect_file().unwrap();

                let ai_config = AiConfig::galaxy_load(&EnvDict::from(&var_space)).err_conv()?;
                let ai_client = AiClient::new(ai_config).err_conv()?;
                let mut message = read_to_string(output.as_path()).owe_data()?;
                let gxl = read_to_string(PathBuf::from("./.run.gxl")).owe_data()?;
                message.push_str("=========== run gxl file ============ \n");
                message.push_str(gxl.as_str());
                println!("Send AI Anaylse ....");
                let ai_response = ai_client
                    .smart_role_request(AiRole::GalactiWard, message.as_str())
                    .await
                    .err_conv()?;
                let response_content = ai_response.content;
                let response_provider = ai_response.provider.to_string();
                println!("AI Response:\nContent: {response_content}\nModel: {response_provider}\n");
            }
        }

        Ok(_) => {
            stop_redirect(redirect);
            return Ok(());
        }
    }
    stop_redirect(redirect);
    process::exit(-1);
}
