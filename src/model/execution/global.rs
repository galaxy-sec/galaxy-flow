use std::{env::current_dir, path::PathBuf};

use orion_common::serde::Configable;
use orion_error::{ErrorConv, ErrorOwe, ErrorWith};
use orion_variate::vars::{ValueDict, ValueType};

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
    let prj_root = find_project_define().unwrap_or(PathBuf::from("UNDEFIN"));
    vars_dict.set("GXL_PRJ_ROOT", format!("{}", prj_root.display()));
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
    format!("{arch}_{os_type }_{ver_major}",)
}

pub fn load_secfile(vars_dict: &mut VarDict) -> ExecResult<()> {
    let env_path = std::env::var("GAL_SEC_FILE_PATH").map(PathBuf::from);
    let default = sec_value_default_path();
    let path = env_path.unwrap_or(default);
    if path.exists() {
        let dict = ValueDict::from_conf(&path).owe_logic()?;
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
        default.save_conf(&path).owe_res()?;
    }
    Ok(())
}

/// 从当前目录开始向上查找 _gal/project.toml 文件
/// 如果找到则返回其绝对路径的PathBuf，未找到则返回None
pub fn find_project_define() -> Option<PathBuf> {
    let mut current_dir = std::env::current_dir().expect("Failed to get current directory");

    loop {
        let project_file = current_dir.join("_gal").join("project.toml");
        if project_file.exists() {
            //let project_root = current_dir.clone();
            return Some(current_dir);
        }

        match current_dir.parent() {
            Some(parent) => current_dir = parent.to_path_buf(),
            None => break, // 已到达根目录
        }
    }

    None
}

#[cfg(test)]
mod tests {

    use crate::{execution::global::find_project_define, util::path::WorkDir};
    use tempfile::TempDir;

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_current_dir() {
        // 创建临时目录
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");
        let _wd = WorkDir::change(temp_dir.path());

        // 调用函数并断言结果
        assert!(find_project_define().is_some())
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_in_parent_dir() {
        // 创建临时目录结构: temp_dir/child/_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let child_dir = temp_dir.path().join("child");
        std::fs::create_dir(&child_dir).expect("Failed to create child dir");
        let gal_dir = temp_dir.path().join("_gal");
        std::fs::create_dir(&gal_dir).expect("Failed to create _gal dir");
        let project_file = gal_dir.join("project.toml");
        std::fs::write(&project_file, "").expect("Failed to create project.toml");

        // 设置当前工作目录为child_dir
        let _wd = WorkDir::change(&child_dir);
        //env::set_current_dir(&child_dir).expect("Failed to set current dir");

        // 调用函数应找到父目录中的文件
        assert!(find_project_define().is_some());
    }

    #[ignore = "change work dir"]
    #[test]
    fn test_find_project_define_not_found() {
        // 创建临时目录，不创建_gal/project.toml
        let temp_dir = TempDir::new().expect("Failed to create temp dir");

        let _wd = WorkDir::change(temp_dir.path());
        // 设置当前工作目录为临时目录
        //env::set_current_dir(temp_dir.path()).expect("Failed to set current dir");

        // 调用函数应返回None
        assert_eq!(find_project_define(), None);
    }
}
