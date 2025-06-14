use std::{env::current_dir, path::PathBuf};

use orion_error::{ErrorConv, ErrorOwe, ErrorWith};
use orion_syspec::{
    types::Configable,
    vars::{ValueDict, ValueType},
};

use crate::{
    traits::Setter,
    var::{SecVar, VarDict},
    ExecResult,
};

use super::dict::{galaxy_dot_path, sec_value_default_path};

pub fn setup_start_vars(vars_dict: &mut VarDict) -> ExecResult<()> {
    vars_dict.set("GXL_OS_SYS", format_os_sys().as_str());

    let start_root = current_dir().owe_sys().want("get current dir")?;
    vars_dict.set("GXL_START_ROOT", start_root.display().to_string());
    Ok(())
}

pub fn setup_gxlrun_vars(vars_dict: &mut VarDict) -> ExecResult<()> {
    let start_root = current_dir().owe_sys().want("get current dir")?;
    vars_dict.set("GXL_CUR_DIR", start_root.display().to_string());
    Ok(())
}

fn get_os_info() -> (String, String, u64) {
    let info = os_info::get();
    let os_type = match info.os_type() {
        os_info::Type::Macos => "macos".to_string(),
        _ => info.os_type().to_string().to_lowercase(),
    };

    let arch = info.architecture().unwrap_or("unknown").to_string();
    let ver_major = match info.version() {
        os_info::Version::Semantic(major, _, _) => *major,
        _ => 0,
    };

    (arch, os_type, ver_major)
}

fn format_os_sys() -> String {
    let (arch, os_type, ver_major) = get_os_info();
    format!("{}_{}_{}", arch, os_type, ver_major)
}

pub fn load_secfile(vars_dict: &mut VarDict) -> ExecResult<()> {
    let env_path = std::env::var("GAL_SEC_FILE_PATH").map(PathBuf::from);
    let default = sec_value_default_path();
    let path = env_path.unwrap_or(default);
    if path.exists() {
        let dict = ValueDict::from_conf(&path).err_conv()?;
        info!(target: "exec","  load {}", path.display());
        for (k, v) in dict.iter() {
            vars_dict.set(
                format!("SEC_{}", k.to_uppercase()),
                SecVar::sec_value(v.to_string()),
            );
        }
    } else {
        let mut default = ValueDict::new();
        default.insert("example_key1", ValueType::from("value"));
        let dot_path = galaxy_dot_path();
        if !dot_path.exists() {
            std::fs::create_dir_all(dot_path).owe_res()?;
        }
        default.save_conf(&path).err_conv()?;
    }
    Ok(())
}
