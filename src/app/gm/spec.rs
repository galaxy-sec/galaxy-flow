use std::path::PathBuf;

use galaxy_flow::err::RunResult;
use orion_error::ErrorConv;
use orion_syspec::module::spec::{make_mod_spec_example, make_mod_spec_new};
use orion_syspec::module::work::{make_modins_example, make_modins_new, RunningModule};
use orion_syspec::system::spec::{make_sys_spec_example, SysModelSpec};
use orion_syspec::system::work::{make_runsystem_example, make_runsystem_new, RunningSystem};
use orion_syspec::types::{Localizable, Persistable};

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

pub async fn do_syspec_cmd(cmd: args::SysSpecCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::SysSpecCmd::Example => {
            let spec = make_sys_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::SysSpecCmd::Create => {
            let spec = make_sys_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::SysSpecCmd::Update => {
            let spec = SysModelSpec::load_from(&current_dir).err_conv()?;
            spec.update_local().await.err_conv()?;
        }
        args::SysSpecCmd::Check => todo!(),
    }
    Ok(())
}
pub async fn do_sysins_cmd(cmd: args::SysInsCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        args::SysInsCmd::Create(sys_args) => {
            let spec = make_runsystem_new(sys_args.repo.as_str());
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        args::SysInsCmd::Example => {
            let spec = make_runsystem_example();
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
