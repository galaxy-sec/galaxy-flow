mod args;
mod spec;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use args::GSysCmd;
use clap::Parser;
use galaxy_flow::err::*;
use spec::{do_sysins_cmd, do_syspec_cmd};

#[tokio::main]
async fn main() {
    use std::process;
    match GxSys::run().await {
        Err(e) => report_rg_error(e),
        Ok(_) => {
            return;
        }
    }
    process::exit(-1);
}

pub struct GxSys {}
impl GxSys {
    pub async fn run() -> RunResult<()> {
        let cmd = GSysCmd::parse();
        debug!("galaxy flow running .....");
        match cmd {
            GSysCmd::Spec(cmd) => {
                do_syspec_cmd(cmd).await?;
            }
            GSysCmd::Inst(cmd) => {
                do_sysins_cmd(cmd).await?;
            }
        }
        Ok(())
    }
}
