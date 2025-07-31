#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::conf::load_gxl_config;
use galaxy_flow::const_val::gxl_const;
use galaxy_flow::err::report_gxl_error;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::model::task_report::task_rc_config::init_redirect_and_parent_task;
use galaxy_flow::runner::{GxlCmd, GxlRunner};
use galaxy_flow::traits::Setter;
use galaxy_flow::util::redirect::stop_redirect;
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init()?;

    // 检查是否请求版本信息

    let mut cmd = GxlCmd::parse();
    // 加载task配置

    configure_run_logging(cmd.log.clone(), cmd.debug);
    load_gxl_config();
    let redirect = init_redirect_and_parent_task(cmd.flow.concat()).await?;
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
    match GxlRunner::run(cmd, var_space, None).await {
        Err(e) => report_gxl_error(e),
        Ok(_) => {
            stop_redirect(redirect)?;
            return Ok(());
        }
    }
    stop_redirect(redirect)?;
    process::exit(-1);
}
