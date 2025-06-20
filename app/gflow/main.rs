#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::report_center::main_task::{create_main_task, get_task_parent_id};
use galaxy_flow::report_center::task_rc_config::load_task_config;
use galaxy_flow::traits::Setter;

use galaxy_flow::err::*;
use galaxy_flow::infra::configure_run_logging;
use galaxy_flow::runner::{GxlCmd, GxlRunner};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    use std::process;

    let mut var_space = VarSpace::sys_init()?;
    let mut cmd = GxlCmd::parse();
    // 加载task配置
    load_task_config().await;

    // 若环境变量中没有设置父id，则将本次任务设置为父任务
    if get_task_parent_id().is_none() {
        let task_name = cmd.flow.concat();
        create_main_task(task_name).await;
    }
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
            return Ok(());
        }
    }
    process::exit(-1);
}
