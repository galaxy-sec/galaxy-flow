use std::path::PathBuf;

use galaxy_flow::err::RunResult;
use orion_error::ErrorConv;
use orion_syspec::system::spec::{make_sys_spec_example, SysModelSpec};
use orion_syspec::system::work::{make_runsystem_example, make_runsystem_new, RunningSystem};
use orion_syspec::types::Persistable;

use crate::args::{self};

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
            let spec = make_runsystem_new(sys_args.repo(), sys_args.path());
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
