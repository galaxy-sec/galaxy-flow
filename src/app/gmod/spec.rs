use galaxy_flow::err::RunResult;
use orion_error::ErrorConv;
use orion_syspec::module::spec::{make_mod_spec_example, make_mod_spec_new};
use orion_syspec::module::work::{make_modins_example, make_modins_new, RunningModule};
use orion_syspec::types::{Localizable, Persistable};
use std::path::PathBuf;

use crate::args::{self};

pub async fn do_modspec_cmd(cmd: args::ModSpecCmd) -> RunResult<()> {
    match cmd {
        args::ModSpecCmd::Create(spec_args) => {
            let spec = make_mod_spec_new(spec_args.name.as_str()).err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::ModSpecCmd::Check => todo!(),
        args::ModSpecCmd::Example => {
            let spec = make_mod_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
    }
    Ok(())
}
pub async fn do_modins_cmd(cmd: args::ModInsCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::ModInsCmd::Example => {
            let spec = make_modins_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::ModInsCmd::Create(spec_args) => {
            let spec = make_modins_new(
                spec_args.name.as_str(),
                "https://e.coding.net/dy-sec/galaxy-open/modspec.git",
            )
            .err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::ModInsCmd::Update => {
            let spec = RunningModule::load_from(&current_dir).err_conv()?;
            spec.update().await.err_conv()?;
        }
        args::ModInsCmd::Local => {
            let spec = RunningModule::load_from(&current_dir).err_conv()?;
            spec.localize().await.err_conv()?;
        }
    }
    Ok(())
}
