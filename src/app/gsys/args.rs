use clap::Parser;
use derive_getters::Getters;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gm")]
#[command(version, about)]
pub enum GSysCmd {
    Example,
    New(NewArgs),
    Load(LoadArgs),
    Update,
    Localize,
}

#[derive(Debug, Args, Getters)]
pub struct NewArgs {
    #[arg(short, long)]
    pub(crate) name: String,
}

#[derive(Debug, Args, Getters)]
pub struct LoadArgs {
    #[arg(short, long)]
    pub(crate) repo: String,
    #[arg(short, long)]
    pub(crate) path: String,
}
