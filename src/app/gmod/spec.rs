use galaxy_flow::err::RunResult;
use galaxy_flow::infra::configure_dfx_logging;
use orion_error::ErrorConv;
use orion_syspec::module::spec::{make_mod_spec_example, make_mod_spec_new};
use orion_syspec::module::target::ModTargetSpec;
use orion_syspec::types::{Localizable, Persistable};
use std::path::PathBuf;

use crate::args::{self};

pub async fn do_mod_cmd(cmd: args::GxModCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::GxModCmd::Example => {
            let spec = make_mod_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./"), None).err_conv()?;
        }
        args::GxModCmd::New(spec_args) => {
            configure_dfx_logging(&spec_args);
            let spec = make_mod_spec_new(spec_args.name.as_str()).err_conv()?;
            spec.save_to(&PathBuf::from("./"), None).err_conv()?;
        }
        args::GxModCmd::Localize(dfx) => {
            configure_dfx_logging(&dfx);
            let spec = ModTargetSpec::load_from(&current_dir).err_conv()?;
            spec.localize(None).await.err_conv()?;
        }
    }
    Ok(())
}
