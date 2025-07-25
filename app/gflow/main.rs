#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::conf::load_gxl_config;
use galaxy_flow::err::report_gxl_error;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::model::task_report::task_rc_config::init_redirect_and_parent_task;
use galaxy_flow::runner::{GxlCmd, GxlRunner};
use galaxy_flow::traits::Setter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init()?;
    let mut cmd = GxlCmd::parse();
    // 加载task配置
    load_gxl_config();
    let redirect = init_redirect_and_parent_task(cmd.flow.concat()).await?;
    configure_run_logging(cmd.log.clone(), cmd.debug);

    debug!("galaxy flow running .....");
    if cmd.conf.is_none() {
        let main_conf = "./_gal/work.gxl";
        cmd.conf = Some(main_conf.to_string());
    }
    var_space
        .global_mut()
        .set("GXL_CMD_ARG", cmd.cmd_arg.clone());
    match GxlRunner::run(cmd, var_space).await {
        Err(e) => report_gxl_error(e),
        Ok(_) => {
            if let Some(mut r) = redirect {
                r.stop();
            }
            return Ok(());
        }
    }

    process::exit(-1);
}
