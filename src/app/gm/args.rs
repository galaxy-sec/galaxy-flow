//use crate::vault::{SysCmd, VaultCmd};
use clap::ArgAction;
use clap::Parser;
use galaxy_flow::runner::GxlCmd;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gm")]
#[command(version, about)]
pub enum GxAdmCmd {
    #[command(subcommand)]
    Prj(PrjCmd),
    Adm(GxlCmd),
    #[command(subcommand)]
    ModSpec(ModSpecCmd),
    #[command(subcommand)]
    ModIns(ModInsCmd),

    #[command(subcommand)]
    SysSpec(SysSpecCmd),
    #[command(subcommand)]
    SysIns(SysInsCmd),
    /// vault cmd
    //#[command(subcommand)]
    //Vault(VaultCmd),
    //#[command(subcommand)]
    //Sys(SysCmd),
    Check,
}

#[derive(Debug, Subcommand)]
pub enum ModSpecCmd {
    Example,
    Create(SpecArgs),
    Check,
}
#[derive(Debug, Subcommand)]
pub enum SysSpecCmd {
    Example,
    Create,
    Update,
    Check,
}

#[derive(Debug, Subcommand)]
pub enum ModInsCmd {
    Example,
    Create(SpecArgs),
    Update,
    Local,
}

#[derive(Debug, Subcommand)]
pub enum SysInsCmd {
    Example,
    Create(SysArgs),
    Update,
    Local,
}

#[derive(Debug, Subcommand)]
pub enum PrjCmd {
    Init,
    RemoteInit(InitArgs),
    Update(PrjArgs),
}

#[derive(Debug, Args)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}
#[derive(Debug, Args)]
pub struct SysArgs {
    #[arg(short, long)]
    pub(crate) repo: String,
}

#[derive(Debug, Args)]
pub struct InitArgs {
    /// chose init tpl  from rg-tpl repo. eg: --tpl open_pages , --tpl rust_prj
    #[arg(short, long, default_value = "simple")]
    pub(crate) tpl: String,
    /// branch or tag for rg-tpl repo. eg: --tag  alpha|develop|beta|release/1.0
    #[arg(short, long, default_value = "stable")]
    pub(crate) channel: String,
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,

    #[arg(long = "repo", default_value = "https://gal-tpl.git")]
    pub repo: String,
    #[arg(long = "log")]
    pub log: Option<String>,
    #[arg(short= 'p', long = "cmd_print" ,action = ArgAction::SetTrue, default_value = "false")]
    pub cmd_print: bool,
}

#[derive(Debug, Args)]
pub struct PrjArgs {
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub(crate) debug: usize,
    /// conf file ; eg: -f ./_gal/prj.gxl
    #[arg(long, default_value = "./_gal/work.gxl")]
    pub(crate) conf_work: String,
    #[arg(long, default_value = "./_gal/adm.gxl")]
    pub(crate) conf_adm: String,
    #[arg(long = "log")]
    pub log: Option<String>,
    #[arg(short= 'q', long = "quiet" ,action = ArgAction::SetFalse , default_value = "true")]
    pub cmd_print: bool,
}

#[derive(Debug, Args)]
pub struct FmtArgs {
    #[arg(short, long, default_value = "info")]
    pub(crate) fmt: String,
}
