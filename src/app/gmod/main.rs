mod args;
mod spec;
//mod vault;

#[macro_use]
extern crate log;
#[macro_use]
extern crate clap;

use crate::args::GxModCmd;
use clap::Parser;
use galaxy_flow::err::*;
use spec::{do_modins_cmd, do_modspec_cmd};

#[tokio::main]
async fn main() {
    use std::process;
    match GxMod::run().await {
        Err(e) => report_rg_error(e),
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
        match cmd {
            GxModCmd::Spec(cmd) => {
                do_modspec_cmd(cmd).await?;
            }
            GxModCmd::Inst(cmd) => {
                do_modins_cmd(cmd).await?;
            }
        }
        Ok(())
    }
}
