#[macro_use]
extern crate log;
extern crate clap;

use clap::Parser;
use std::path::Path;

use galaxy_flow::err::*;
use galaxy_flow::infra::configure_flow_logging;
use galaxy_flow::runner::{GxlCmd, GxlRunner};

fn main() -> anyhow::Result<()> {
    use std::process;
    let mut cmd = GxlCmd::parse();
    configure_flow_logging(cmd.log.clone(), cmd.debug);
    //log_init(&LogConf::alpha())?;
    debug!("galaxy flow running .....");
    if cmd.conf.is_none() {
        let v1_conf = "./_gal/prj.gxl";
        let v2_conf = "./_gal/work.gxl";
        if Path::new(v2_conf).exists() {
            cmd.conf = Some(v2_conf.to_string());
        } else if Path::new(v1_conf).exists() {
            cmd.conf = Some(v1_conf.to_string());
            println!("warning: please use work.gxl !");
        }
    }
    match GxlRunner::run(cmd) {
        Err(e) => report_rg_error(e),
        Ok(_) => {
            return Ok(());
        }
    }
    process::exit(-1);
}
