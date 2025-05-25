use clap::Parser;

#[derive(Parser, Debug)] // requires `derive` feature
#[command(version, about, long_about = None)]
pub struct RgCmd {
    /// env name ; eg: -e dev
    #[arg(short = 'e', long = "env", default_value = "default")]
    pub env: String,
    /// flow name ; eg: conf,test,package
    pub flow: Vec<String>,
    /// debug level ; eg: -d 1
    #[arg(short = 'd', long = "debug", default_value = "0")]
    pub debug: usize,
    /// conf file ; eg: -f ./_gal/prj.gxl
    #[arg(short = 'f', long = "conf", default_value = "./_gal/work.gxl")]
    pub conf: String,
}
