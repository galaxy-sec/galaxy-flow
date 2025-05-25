use clap::Parser;
use derive_getters::Getters;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gm")]
#[command(version, about)]
pub enum GSysCmd {
    #[command(subcommand)]
    Spec(SysSpecCmd),
    #[command(subcommand)]
    Inst(SysInsCmd),
}

#[derive(Debug, Subcommand)]
pub enum SysSpecCmd {
    Example,
    Create,
    Update,
    Check,
}

#[derive(Debug, Subcommand)]
pub enum SysInsCmd {
    Example,
    New(SysInsArgs),
    Update,
    Local,
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
