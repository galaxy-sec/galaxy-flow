#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::cmd::gxl_cmd::GFlowCmd;
use galaxy_flow::conf::load_gxl_config;
use galaxy_flow::const_val::gxl_const;
use galaxy_flow::err::{RunResult, report_gxl_error};
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::model::task_report::task_rc_config::init_redirect_and_parent_task;
use galaxy_flow::runner::GxlRunner;
use galaxy_flow::traits::Setter;
use galaxy_flow::util::diagnose::ai_diagnose;
use galaxy_flow::util::redirect::stop_redirect;
use orion_ai::GlobalFunctionRegistry;
use orion_error::{ErrorConv, UvsBizFrom};
use std::env;

#[tokio::main]
async fn main() -> RunResult<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init().err_conv()?;

    // 检查是否请求版本信息

    let mut cmd = GFlowCmd::parse();
    // 加载task配置

    configure_run_logging(cmd.log.clone(), cmd.debug);
    load_gxl_config();

    // 初始化全局函数注册表
    if let Err(e) = GlobalFunctionRegistry::initialize() {
        eprintln!("Failed to initialize global function registry: {}", e);
        return Err(galaxy_flow::err::RunReason::from_biz(format!(
            "Global function registry initialization failed: {}",
            e
        ))
        .into());
    }
    println!("✅ 全局函数注册表初始化完成");

    let redirect = init_redirect_and_parent_task(cmd.flows.join(","), cmd.ai)
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
        .set(gxl_const::CMD_ARG, cmd.cmd_args.join(" "));
    var_space
        .global_mut()
        .set(gxl_const::CMD_DRYRUN, cmd.dryrun);
    var_space
        .global_mut()
        .set(gxl_const::CMD_MODUP, cmd.mod_update);
    if cmd.list_cmd().is_empty() {
        GxlRunner::info(cmd.conf.clone(), var_space).await?;
    } else {
        for cmd in cmd.list_cmd() {
            match GxlRunner::run(cmd.clone(), var_space.clone(), None).await {
                Err(e) => {
                    report_gxl_error(e);
                    if cmd.ai
                        && let Err(e) = ai_diagnose(&var_space).await
                    {
                        report_gxl_error(e);
                    }
                }

                Ok(_) => {
                    let _ = stop_redirect(redirect);
                    return Ok(());
                }
            }
        }
    }
    let _ = stop_redirect(redirect);
    process::exit(-1);
}
