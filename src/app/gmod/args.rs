//use crate::vault::{SysCmd, VaultCmd};
use clap::Parser;
use derive_getters::Getters;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gm")]
#[command(version, about)]
pub enum GxModCmd {
    #[command(subcommand)]
    Spec(ModSpecCmd),
    #[command(subcommand)]
    Inst(ModInsCmd),
}

#[derive(Debug, Subcommand)]
pub enum ModSpecCmd {
    Example,
    Create(SpecArgs),
    Check,
}

#[derive(Debug, Subcommand)]
pub enum ModInsCmd {
    Example,
    Create(SpecArgs),
    Update,
    Local,
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}
