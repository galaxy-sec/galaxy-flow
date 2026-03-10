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

use orion_error::{ErrorOwe, ErrorWith, ToStructError};

use crate::err::{RunReason, RunResult};

const SELF_UPDATE_BINARIES: &[&str] = &["gx"];

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
    Version::parse(raw)
        .owe_data()
        .want("parse semantic version")
        .with(("version", v))
}

pub fn is_remote_newer(current: &str, remote: &str) -> RunResult<bool> {
    Ok(normalize_version(remote)? > normalize_version(current)?)
}

pub fn verify_sha256(file: &Path, expect: &str) -> RunResult<()> {
    let mut f = fs::File::open(file)
        .owe_res()
        .want("open file for sha256")
        .with(("path", file))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = f
            .read(&mut buf)
            .owe_res()
            .want("read file for sha256")
            .with(("path", file))?;
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
        .want("verify package sha256")
        .with(("path", file))
        .with(("expect", expect))
        .with(("got", got.as_str()))
        .with_detail(format!(
            "expect={expect}, got={got}, file={}",
            file.display()
        )))
}

pub fn extract_tar_gz(archive: &Path, dest: &Path) -> RunResult<()> {
    fs::create_dir_all(dest)
        .owe_res()
        .want("create extract destination")
        .with(("path", dest))?;
    let file = fs::File::open(archive)
        .owe_res()
        .want("open package archive")
        .with(("path", archive))?;
    let gz = GzDecoder::new(file);
    let mut tar = Archive::new(gz);
    tar.unpack(dest)
        .owe_res()
        .want("extract package archive")
        .with(("archive", archive))
        .with(("dest", dest))
}

pub fn find_binary(root: &Path, bin_name: &str) -> RunResult<PathBuf> {
    let mut matches = Vec::new();
    for item in WalkDir::new(root).follow_links(false).into_iter() {
        let item = item
            .owe_res()
            .want("walk package files")
            .with(("root", root))?;
        if item.path().strip_prefix(root).is_err() {
            continue;
        }
        if item.file_type().is_symlink() {
            continue;
        }
        if item.file_type().is_file() && item.path().file_name() == Some(OsStr::new(bin_name)) {
            matches.push(item.path().to_path_buf());
        }
    }
    if matches.is_empty() {
        return Err(RunReason::Exec("binary not found in package".into())
            .to_err()
            .with_detail(format!("bin={bin_name}, root={}", root.display())));
    }
    if matches.len() > 1 {
        return Err(RunReason::Exec("multiple binaries found in package".into())
            .to_err()
            .with_detail(format!(
                "bin={bin_name}, root={}, count={}",
                root.display(),
                matches.len()
            )));
    }
    Ok(matches.remove(0))
}

pub fn install_dir_from_current_exe() -> RunResult<PathBuf> {
    let exe = std::env::current_exe()
        .owe_sys()
        .want("resolve current executable path")?;
    exe.parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| RunReason::Exec("cannot resolve install dir".into()).to_err())
        .want("resolve install dir from current executable")
        .with(("exe", exe.as_path()))
}

pub fn backup_and_replace(install_dir: &Path, backup_dir: &Path, new_gx: &Path) -> RunResult<()> {
    fs::create_dir_all(backup_dir)
        .owe_res()
        .want("create backup directory")
        .with(("path", backup_dir))?;
    let new_bins = [new_gx];

    for (idx, bin) in SELF_UPDATE_BINARIES.iter().enumerate() {
        let live_path = install_dir.join(bin_name(bin));
        let backup_path = backup_dir.join(bin_name(bin));
        let stage_path = install_dir.join(format!("{}.new", bin_name(bin)));

        copy_file(&live_path, &backup_path)?;
        copy_file(new_bins[idx], &stage_path)?;
        replace_file(&stage_path, &live_path)?;
    }
    Ok(())
}

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
    let status = Command::new(bin)
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
    let mut input = fs::File::open(src)
        .owe_res()
        .want("open source file for copy")
        .with(("src", src))
        .with(("dst", dst))?;
    let mut output = fs::File::create(dst)
        .owe_res()
        .want("create destination file for copy")
        .with(("src", src))
        .with(("dst", dst))?;
    copy(&mut input, &mut output)
        .owe_res()
        .want("copy file bytes")
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

fn replace_file(src: &Path, dst: &Path) -> RunResult<()> {
    #[cfg(windows)]
    if dst.exists() {
        fs::remove_file(dst)
            .owe_res()
            .want("remove old destination file before replace")
            .with(("dst", dst))?;
    }
    fs::rename(src, dst)
        .owe_res()
        .want("replace file")
        .with(("src", src))
        .with(("dst", dst))
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;

    use tempfile::tempdir;

    use super::{
        SELF_UPDATE_BINARIES, find_binary, is_remote_newer, normalize_version, verify_sha256,
    };

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

    #[test]
    fn find_binary_rejects_multiple_matches() {
        let dir = tempdir().expect("tmp dir");
        let d1 = dir.path().join("a");
        let d2 = dir.path().join("b");
        fs::create_dir_all(&d1).expect("mkdir a");
        fs::create_dir_all(&d2).expect("mkdir b");
        fs::write(d1.join("gx"), b"x").expect("write gx a");
        fs::write(d2.join("gx"), b"y").expect("write gx b");
        assert!(find_binary(dir.path(), "gx").is_err());
    }

    #[test]
    fn self_update_binary_list_includes_gx() {
        assert_eq!(SELF_UPDATE_BINARIES, ["gx"]);
    }
}
