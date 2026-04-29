use std::fs;
use std::path::Path;

use orion_error::ErrorWith;
use orion_error::UvsReason;
use orion_error::compat_traits::ErrorOweBase;
use orion_error::traits_ext::ToStructError;

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
        .owe(UvsReason::resource_error().into())
        .doing("run binary version command")
        .with_context(("bin", bin))?;
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
            .owe(UvsReason::resource_error().into())
            .doing("create copy destination parent")
            .with_context(("path", parent))?;
    }
    fs::copy(src, dst)
        .owe(UvsReason::resource_error().into())
        .doing("copy file")
        .with_context(("src", src))
        .with_context(("dst", dst))?;
    let perm = fs::metadata(src)
        .owe(UvsReason::resource_error().into())
        .doing("read source file permissions")
        .with_context(("src", src))?
        .permissions();
    fs::set_permissions(dst, perm)
        .owe(UvsReason::resource_error().into())
        .doing("set destination file permissions")
        .with_context(("src", src))
        .with_context(("dst", dst))?;
    Ok(())
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}
