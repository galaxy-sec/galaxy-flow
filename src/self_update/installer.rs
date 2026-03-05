use std::ffi::OsStr;
use std::fs;
use std::io::{Read, copy};
use std::path::{Path, PathBuf};
use std::process::Command;

use flate2::read::GzDecoder;
use semver::Version;
use sha2::{Digest, Sha256};
use tar::Archive;
use walkdir::WalkDir;

use orion_error::ToStructError;

use crate::err::{RunReason, RunResult};

pub fn detect_target_triple() -> RunResult<String> {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = match (arch, os) {
        ("x86_64", "linux") => "x86_64-unknown-linux-gnu",
        ("aarch64", "macos") => "aarch64-apple-darwin",
        ("x86_64", "macos") => "x86_64-apple-darwin",
        _ => {
            return Err(RunReason::Args("unsupported platform".into())
                .to_err()
                .with_detail(format!("arch={arch}, os={os}")));
        }
    };
    Ok(target.to_string())
}

pub fn normalize_version(v: &str) -> RunResult<Version> {
    let raw = v.trim().trim_start_matches('v');
    Version::parse(raw).map_err(parse_err)
}

pub fn is_remote_newer(current: &str, remote: &str) -> RunResult<bool> {
    Ok(normalize_version(remote)? > normalize_version(current)?)
}

pub fn verify_sha256(file: &Path, expect: &str) -> RunResult<()> {
    let mut f = fs::File::open(file).map_err(io_err)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f.read(&mut buf).map_err(io_err)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let got = format!("{:x}", hasher.finalize());
    if got.eq_ignore_ascii_case(expect.trim()) {
        return Ok(());
    }
    Err(RunReason::Exec("sha256 mismatch".into())
        .to_err()
        .with_detail(format!(
            "expect={expect}, got={got}, file={}",
            file.display()
        )))
}

pub fn extract_tar_gz(archive: &Path, dest: &Path) -> RunResult<()> {
    fs::create_dir_all(dest).map_err(io_err)?;
    let file = fs::File::open(archive).map_err(io_err)?;
    let gz = GzDecoder::new(file);
    let mut tar = Archive::new(gz);
    tar.unpack(dest).map_err(io_err)
}

pub fn find_binary(root: &Path, bin_name: &str) -> RunResult<PathBuf> {
    for item in WalkDir::new(root).follow_links(true).into_iter() {
        let item = item.map_err(walk_err)?;
        if item.file_type().is_file() && item.path().file_name() == Some(OsStr::new(bin_name)) {
            return Ok(item.path().to_path_buf());
        }
    }
    Err(RunReason::Exec("binary not found in package".into())
        .to_err()
        .with_detail(format!("bin={bin_name}, root={}", root.display())))
}

pub fn install_dir_from_current_exe() -> RunResult<PathBuf> {
    let exe = std::env::current_exe().map_err(io_err)?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| RunReason::Exec("cannot resolve install dir".into()).to_err())
}

pub fn backup_and_replace(
    install_dir: &Path,
    backup_dir: &Path,
    new_gprj: &Path,
    new_gflow: &Path,
) -> RunResult<()> {
    fs::create_dir_all(backup_dir).map_err(io_err)?;
    let gprj_path = install_dir.join(bin_name("gprj"));
    let gflow_path = install_dir.join(bin_name("gflow"));

    copy_file(&gprj_path, &backup_dir.join(bin_name("gprj")))?;
    copy_file(&gflow_path, &backup_dir.join(bin_name("gflow")))?;

    let gprj_stage = install_dir.join(format!("{}.new", bin_name("gprj")));
    let gflow_stage = install_dir.join(format!("{}.new", bin_name("gflow")));
    copy_file(new_gprj, &gprj_stage)?;
    copy_file(new_gflow, &gflow_stage)?;

    replace_file(&gprj_stage, &gprj_path)?;
    replace_file(&gflow_stage, &gflow_path)?;
    Ok(())
}

pub fn rollback(install_dir: &Path, backup_dir: &Path) -> RunResult<()> {
    let gprj_path = install_dir.join(bin_name("gprj"));
    let gflow_path = install_dir.join(bin_name("gflow"));
    let b_gprj = backup_dir.join(bin_name("gprj"));
    let b_gflow = backup_dir.join(bin_name("gflow"));
    if !b_gprj.exists() || !b_gflow.exists() {
        return Err(RunReason::Args("backup is incomplete".into())
            .to_err()
            .with_detail(format!("backup_dir={}", backup_dir.display())));
    }
    copy_file(&b_gprj, &gprj_path)?;
    copy_file(&b_gflow, &gflow_path)?;
    Ok(())
}

pub fn health_check(install_dir: &Path) -> RunResult<()> {
    exec_version(&install_dir.join(bin_name("gprj")))?;
    exec_version(&install_dir.join(bin_name("gflow")))?;
    Ok(())
}

fn exec_version(bin: &Path) -> RunResult<()> {
    let status = Command::new(bin)
        .arg("--version")
        .status()
        .map_err(io_err)?;
    if status.success() {
        return Ok(());
    }
    Err(RunReason::Exec("health check failed".into())
        .to_err()
        .with_detail(format!("{} --version exit={status}", bin.display())))
}

fn copy_file(src: &Path, dst: &Path) -> RunResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(io_err)?;
    }
    let mut input = fs::File::open(src).map_err(io_err)?;
    let mut output = fs::File::create(dst).map_err(io_err)?;
    copy(&mut input, &mut output).map_err(io_err)?;
    let perm = fs::metadata(src).map_err(io_err)?.permissions();
    fs::set_permissions(dst, perm).map_err(io_err)?;
    Ok(())
}

fn replace_file(src: &Path, dst: &Path) -> RunResult<()> {
    #[cfg(windows)]
    if dst.exists() {
        fs::remove_file(dst).map_err(io_err)?;
    }
    fs::rename(src, dst).map_err(io_err)
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn io_err(err: std::io::Error) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

fn parse_err(err: impl std::fmt::Display) -> crate::err::RunError {
    RunReason::Args(err.to_string()).to_err()
}

fn walk_err(err: walkdir::Error) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use tempfile::tempdir;

    use super::{is_remote_newer, normalize_version, verify_sha256};

    #[test]
    fn normalize_version_accepts_v_prefix() {
        let v = normalize_version(" v1.2.3 ").expect("parse version");
        assert_eq!(v.to_string(), "1.2.3");
    }

    #[test]
    fn is_remote_newer_handles_prerelease() {
        assert!(is_remote_newer("0.12.3", "0.12.4").expect("compare versions"));
        assert!(!is_remote_newer("0.12.4", "0.12.4-pre.1").expect("compare versions"));
    }

    #[test]
    fn verify_sha256_succeeds_and_fails() {
        let dir = tempdir().expect("tmp dir");
        let file = dir.path().join("data.txt");
        let mut f = fs::File::create(&file).expect("create file");
        writeln!(f, "hello-sha256").expect("write file");
        drop(f);

        let ok = "0f7f071ced9d0deec80c6011cada66dc2af6212d5542e895801898a555da5b70";
        verify_sha256(&file, ok).expect("sha ok");
        assert!(verify_sha256(&file, "0000").is_err());
    }
}
