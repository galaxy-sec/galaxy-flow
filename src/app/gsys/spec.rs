use std::path::PathBuf;

use galaxy_flow::err::RunResult;
use orion_error::ErrorConv;
use orion_syspec::addr::GitAddr;
use orion_syspec::system::refs::SysModelSpecRef;
use orion_syspec::system::spec::{make_sys_spec_example, make_sys_spec_new, SysModelSpec};
use orion_syspec::types::{AsyncUpdateable, Localizable};

use crate::args::GSysCmd;

pub async fn do_sys_cmd(cmd: GSysCmd) -> RunResult<()> {
    let current_dir = std::env::current_dir().expect("无法获取当前目录");
    match cmd {
        GSysCmd::Example => {
            let spec = make_sys_spec_example().err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        GSysCmd::New(args) => {
            let spec = make_sys_spec_new(args.name(), "https://").err_conv()?;
            spec.save_to(&PathBuf::from("./")).err_conv()?;
        }
        GSysCmd::Load(args) => {
            let target = args.path();
            let spec_ref = SysModelSpecRef::from(target, GitAddr::from(args.repo()).path(target));
            spec_ref.update_local(&current_dir).await.err_conv()?;
        }
        GSysCmd::Update => {
            let spec = SysModelSpec::load_from(&current_dir).err_conv()?;
            spec.update_local().await.err_conv()?;
        }
        GSysCmd::Localize => {
            let spec = SysModelSpec::load_from(&current_dir).err_conv()?;
            spec.localize(None).await.err_conv()?;
        }
    }
    Ok(())
}
