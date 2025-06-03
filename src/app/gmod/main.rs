mod args;
mod spec;
//mod vault;

extern crate log;
#[macro_use]
extern crate clap;

use crate::args::GxModCmd;
use clap::Parser;
use galaxy_flow::err::*;
use spec::do_mod_cmd;

#[tokio::main]
async fn main() {
    use std::process;
    match GxMod::run().await {
        Err(e) => report_gxl_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxMod {}
impl GxMod {
    pub async fn run() -> RunResult<()> {
        let cmd = GxModCmd::parse();
        do_mod_cmd(cmd).await?;
        Ok(())
    }
}
