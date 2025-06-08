use clap::Parser;
use derive_getters::Getters;
use galaxy_flow::infra::DfxArgsGetter;

#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "gm")]
#[command(version, about)]
pub enum GSysCmd {
    Example,
    New(NewArgs),
    Load(LoadArgs),
    Update(DfxArgs),
    Localize(DfxArgs),
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
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}

#[derive(Debug, Args, Getters)]
pub struct DfxArgs {
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// config log ; eg: --log  cmd=debug,parse=info
    #[arg(long = "log")]
    pub log: Option<String>,
}

impl DfxArgsGetter for DfxArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}

impl DfxArgsGetter for LoadArgs {
    fn debug_level(&self) -> usize {
        self.debug
    }

    fn log_setting(&self) -> Option<String> {
        self.log.clone()
    }
}
