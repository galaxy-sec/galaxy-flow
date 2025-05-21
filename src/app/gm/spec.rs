use std::path::PathBuf;

use galaxy_flow::err::RunResult;
use orion_error::ErrorConv;
use orion_syspec::mod_run::{make_modins_example, RunningModule};
use orion_syspec::modul::make_mod_spec_example;
use orion_syspec::sys_run::RunningSystem;
use orion_syspec::system::make_sys_spec_example;
use orion_syspec::types::{Localizable, Persistable};

use crate::args;

pub async fn do_modspec_cmd(cmd: args::ModSpecCmd) -> RunResult<()> {
    match cmd {
        args::ModSpecCmd::Crate => {
            let spec = make_mod_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::ModSpecCmd::Check => todo!(),
    }
    Ok(())
}
pub async fn do_modins_cmd(cmd: args::ModInsCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::ModInsCmd::Crate => {
            let spec = make_modins_example().err_conv()?;
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

pub async fn do_syspec_cmd(cmd: args::SysSpecCmd) -> RunResult<()> {
    match cmd {
        args::SysSpecCmd::Crate => {
            let spec = make_sys_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::SysSpecCmd::Check => todo!(),
    }
    Ok(())
}
pub async fn do_sysins_cmd(cmd: args::SysInsCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::SysInsCmd::Crate => {
            let spec = make_modins_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::SysInsCmd::Update => {
            let spec = RunningSystem::load_from(&current_dir).err_conv()?;
            spec.update().await.err_conv()?;
        }
        args::SysInsCmd::Local => {
            let spec = RunningSystem::load_from(&current_dir).err_conv()?;
            spec.localize().await.err_conv()?;
        }
    }
    Ok(())
}
