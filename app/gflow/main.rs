#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use galaxy_flow::conf::load_gxl_config;
use galaxy_flow::const_val::gxl_const;
use galaxy_flow::execution::VarSpace;
use galaxy_flow::task_report::main_task::{create_main_task, get_task_parent_id};
use galaxy_flow::task_report::task_rc_config::report_enable;
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
    load_gxl_config();

    // 若环境变量中没有设置父id，则将本次任务设置为父任务
    if get_task_parent_id().is_none() {
        // 使用代码块限制读锁的作用域，确保在调用create_main_task之前已经释放锁
        if report_enable().await {
            let task_name = cmd.flow.concat();
            create_main_task(task_name).await;
        }
    }
    configure_run_logging(cmd.log.clone(), cmd.debug);
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
        .set(gxl_const::CMD_DRYRUN, cmd.dryrun.clone());
    var_space
        .global_mut()
        .set(gxl_const::CMD_MODUP, cmd.mod_update.clone());
    match GxlRunner::run(cmd, var_space).await {
        Err(e) => report_gxl_error(e),
        Ok(_) => {
            return Ok(());
        }
    }
    process::exit(-1);
}
