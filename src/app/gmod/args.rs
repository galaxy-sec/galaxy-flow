//use crate::vault::{SysCmd, VaultCmd};
use clap::Parser;
use derive_getters::Getters;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gmod")]
#[command(version, about)]
pub enum GxModCmd {
    Example,
    New(SpecArgs),
    Local,
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}
