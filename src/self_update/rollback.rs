use std::fs;
use std::path::Path;

use orion_error::{ErrorOwe, ErrorWith, ToStructError};

use crate::err::{RunReason, RunResult};

const SELF_UPDATE_BINARIES: &[&str] = &["gx"];

pub fn rollback(install_dir: &Path, backup_dir: &Path) -> RunResult<()> {
    for bin in SELF_UPDATE_BINARIES {
        let live_path = install_dir.join(bin_name(bin));
        let backup_path = backup_dir.join(bin_name(bin));
        if !backup_path.exists() {
            return Err(RunReason::Args("backup is incomplete".into())
                .to_err()
                .with_detail(format!(
                    "backup_dir={}, missing={}",
                    backup_dir.display(),
                    backup_path.display()
                )));
        }
        copy_file(&backup_path, &live_path)?;
    }
    Ok(())
}

pub fn health_check(install_dir: &Path) -> RunResult<()> {
    for bin in SELF_UPDATE_BINARIES {
        exec_version(&install_dir.join(bin_name(bin)))?;
    }
    Ok(())
}

fn exec_version(bin: &Path) -> RunResult<()> {
    let status = std::process::Command::new(bin)
        .arg("--version")
        .status()
        .owe_res()
        .want("run binary version command")
        .with(("bin", bin))?;
    if status.success() {
        return Ok(());
    }
    Err(RunReason::Exec("health check failed".into())
        .to_err()
        .with_detail(format!("{} --version exit={status}", bin.display())))
}

fn copy_file(src: &Path, dst: &Path) -> RunResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .owe_res()
            .want("create copy destination parent")
            .with(("path", parent))?;
    }
    fs::copy(src, dst)
        .owe_res()
        .want("copy file")
        .with(("src", src))
        .with(("dst", dst))?;
    let perm = fs::metadata(src)
        .owe_res()
        .want("read source file permissions")
        .with(("src", src))?
        .permissions();
    fs::set_permissions(dst, perm)
        .owe_res()
        .want("set destination file permissions")
        .with(("src", src))
        .with(("dst", dst))?;
    Ok(())
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
