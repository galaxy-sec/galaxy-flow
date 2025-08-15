//use crate::vault::{SysCmd, VaultCmd};
use clap::ArgAction;
use clap::Parser;
use derive_getters::Getters;
use galaxy_flow::runner::GxlCmd;

#[derive(Debug, Parser, Clone)] // requires `derive` feature
#[command(name = "gprj adm")]
#[command(version, about)]
pub enum GxAdmCmd {
    #[command(subcommand)]
    Init(InitCmd),
    #[command(subcommand)]
    Update(UpdateCmd),
    Adm(GxlCmd),
    #[command(subcommand)]
    Conf(ConfCmd),
    Check,
}

#[derive(Debug, Subcommand, Clone)]
pub enum InitCmd {
    /// init galaxy env
    Env,
    /// init project with local mod ;
    PrjWithLocal,
    /// init project with remote mod ;
    Prj(InitArgs),
}
#[derive(Debug, Subcommand, Clone)]
pub enum UpdateCmd {
    Mod(PrjArgs),
}

#[derive(Debug, Subcommand, Clone)]
pub enum ConfCmd {
    Init(ConfInitArgs),
}

#[derive(Debug, Args, Getters)]
pub struct SpecArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}
#[derive(Debug, Args, Getters)]
pub struct SySpecArgs {
    #[arg(short, long)]
    pub(crate) repo: String,
}

#[derive(Debug, Args, Getters)]
pub struct SysInsArgs {
    #[arg(short, long)]
    pub(crate) repo: String,
    #[arg(short, long)]
    pub(crate) path: String,
}

#[derive(Debug, Args, Getters, Clone)]
pub struct InitArgs {
    /// chose init tpl  from rg-tpl repo. eg: --tpl open_pages , --tpl rust_prj
    #[arg(short, long, default_value = "simple")]
    pub(crate) tpl: String,
    /// branch or tag for rg-tpl repo. eg: --tag  alpha|develop|beta|release/1.0
    #[arg(short, long)]
    pub(crate) branch: Option<String>,

    #[arg(short, long)]
    pub(crate) tag: Option<String>,
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

#[derive(Debug, Args, Getters, Clone)]
pub struct ConfInitArgs {
    #[arg(short = 'r', long = "remote", default_value = "false")]
    pub remote: bool,
}

#[derive(Debug, Args, Clone)]
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
