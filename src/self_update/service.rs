use std::path::{Path, PathBuf};
use std::process::Command;

use chrono::Utc;
use serde::Serialize;

use orion_error::{ErrorOwe, ErrorWith, ToStructError};

use crate::err::{RunReason, RunResult};

use super::client::SelfUpdateClient;
use super::installer;
use super::model::{
    CheckResult, ManifestAsset, ReleaseChannel, SelfUpdateState, StatusResult, UpdateResult,
};
use super::storage::SelfUpdateStorage;

const MANIFEST_BASE_URL_STABLE: &str =
    "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/main/updates/stable";
const MANIFEST_BASE_URL_ALPHA: &str =
    "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/alpha/updates/alpha";
const MANIFEST_BASE_URL_BETA: &str =
    "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/beta/updates/beta";

#[derive(Clone, Debug, Default)]
pub struct CheckRequest {
    pub channel: ReleaseChannel,
}

#[derive(Clone, Debug, Default)]
pub struct UpdateRequest {
    pub channel: ReleaseChannel,
    pub to_version: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[derive(Clone, Debug)]
pub struct SelfUpdateService {
    storage: SelfUpdateStorage,
    client: SelfUpdateClient,
}

impl SelfUpdateService {
    pub fn new() -> RunResult<Self> {
        Ok(Self {
            storage: SelfUpdateStorage::new()?,
            client: SelfUpdateClient::default(),
        })
    }

    pub fn status(&self) -> RunResult<StatusResult> {
        let install_dir = installer::install_dir_from_current_exe()?;
        let mut state = self.storage.load_state()?;
        state.current_version = Some(env!("CARGO_PKG_VERSION").to_string());
        Ok(StatusResult {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            install_dir,
            state,
        })
    }

    pub async fn check(&self, req: CheckRequest) -> RunResult<CheckResult> {
        let _lock = self.storage.acquire_lock()?;
        let mut state = self.storage.load_state()?;
        let channel = req.channel;
        let temp_dir = TempDirGuard::new("check")?;
        let manifest_base_url = manifest_base_url(channel);
        let manifest = self
            .client
            .fetch_manifest(manifest_base_url, channel, temp_dir.path())
            .await
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    None,
                    "check_failed",
                    &e.to_string(),
                );
            })?;

        let current = env!("CARGO_PKG_VERSION").to_string();
        let has_update =
            installer::is_remote_newer(&current, &manifest.version).inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(manifest.version.clone()),
                    "check_failed",
                    &e.to_string(),
                );
            })?;

        state.last_checked_at = Some(now_text());
        state.last_channel = Some(channel);
        state.last_remote_version = Some(manifest.version.clone());
        state.last_result = Some(if has_update {
            "update_available".to_string()
        } else {
            "up_to_date".to_string()
        });
        state.last_error = None;
        self.storage.save_state(&state)?;

        Ok(CheckResult {
            channel,
            current_version: current,
            remote_version: manifest.version,
            has_update,
        })
    }

    pub async fn update(&self, req: UpdateRequest) -> RunResult<UpdateResult> {
        let _lock = self.storage.acquire_lock()?;
        let mut state = self.storage.load_state()?;
        let channel = req.channel;
        let current = env!("CARGO_PKG_VERSION").to_string();

        let temp_dir = TempDirGuard::new("update")?;
        let manifest_base_url = manifest_base_url(channel);

        let manifest = self
            .client
            .fetch_manifest(manifest_base_url, channel, temp_dir.path())
            .await
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    None,
                    "update_failed",
                    &e.to_string(),
                );
            })?;
        let remote = manifest.version.clone();
        if let Some(expect) = &req.to_version
            && expect.trim() != remote
        {
            let err = RunReason::Args("target version mismatch".into())
                .to_err()
                .want("validate target update version")
                .with(("expect", expect.as_str()))
                .with(("manifest", remote.as_str()))
                .with_detail(format!("expect={expect}, manifest={remote}"));
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &err.to_string(),
            );
            return Err(err);
        }

        let has_update = if req.force {
            true
        } else {
            installer::is_remote_newer(&current, &remote).inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?
        };
        if !has_update {
            state.last_checked_at = Some(now_text());
            state.last_channel = Some(channel);
            state.last_remote_version = Some(remote.clone());
            state.last_result = Some("up_to_date".to_string());
            state.last_error = None;
            self.storage.save_state(&state)?;
            return Ok(UpdateResult {
                channel,
                from_version: current,
                to_version: remote,
                backup_id: None,
                updated: false,
            });
        }
        if req.dry_run {
            state.last_checked_at = Some(now_text());
            state.last_channel = Some(channel);
            state.last_remote_version = Some(remote.clone());
            state.last_result = Some("dry_run".to_string());
            state.last_error = None;
            self.storage.save_state(&state)?;
            return Ok(UpdateResult {
                channel,
                from_version: current,
                to_version: remote,
                backup_id: None,
                updated: false,
            });
        }
        if !req.yes {
            let err = RunReason::Args("confirmation required".into())
                .to_err()
                .want("confirm update apply")
                .with(("channel", channel.as_str()))
                .with(("remote", remote.as_str()))
                .with_detail("rerun with --yes to apply update, or use --dry-run");
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_skipped_need_confirm",
                &err.to_string(),
            );
            return Err(err);
        }

        let target = installer::detect_target_triple().inspect_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &e.to_string(),
            );
        })?;
        let asset = manifest
            .assets
            .get(&target)
            .ok_or_else(|| {
                RunReason::Exec("asset for target not found".into())
                    .to_err()
                    .want("select package asset for target")
                    .with(("target", target.as_str()))
                    .with_detail(format!("target={target}"))
            })
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?;

        let asset_file = temp_dir.path().join(format!("update-{target}.tar.gz"));
        self.client
            .download_to_path(&asset.url, &asset_file)
            .await
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?;
        installer::verify_sha256(&asset_file, &asset.sha256).inspect_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &e.to_string(),
            );
        })?;

        let unpack_dir = temp_dir.path().join("unpack");
        installer::extract_tar_gz(&asset_file, &unpack_dir).inspect_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &e.to_string(),
            );
        })?;
        let new_gprj =
            installer::find_binary(&unpack_dir, bin_name("gprj").as_str()).inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?;
        let new_gflow = installer::find_binary(&unpack_dir, bin_name("gflow").as_str())
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?;

        let install_dir = installer::install_dir_from_current_exe().inspect_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(remote.clone()),
                "update_failed",
                &e.to_string(),
            );
        })?;
        let backup_id = now_id();
        let backup_dir = self
            .storage
            .create_backup_dir(&backup_id)
            .inspect_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
            })?;

        let install_res =
            installer::backup_and_replace(&install_dir, &backup_dir, &new_gprj, &new_gflow)
                .and_then(|_| installer::health_check(&install_dir));

        if let Err(e) = install_res {
            let _ = installer::rollback(&install_dir, &backup_dir);
            state.last_checked_at = Some(now_text());
            state.last_channel = Some(channel);
            state.last_remote_version = Some(remote.clone());
            state.last_result = Some("update_failed_rollback".to_string());
            state.last_error = Some(e.to_string());
            self.storage.save_state(&state)?;
            return Err(e);
        }

        state.last_checked_at = Some(now_text());
        state.last_channel = Some(channel);
        state.last_remote_version = Some(remote.clone());
        state.last_result = Some("updated".to_string());
        state.last_error = None;
        state.current_version = Some(remote.clone());
        state.installed_at = Some(now_text());
        state.last_backup_id = Some(backup_id.clone());
        self.storage.save_state(&state)?;

        Ok(UpdateResult {
            channel,
            from_version: current,
            to_version: remote,
            backup_id: Some(backup_id),
            updated: true,
        })
    }

    pub fn rollback(&self, id: Option<&str>) -> RunResult<UpdateResult> {
        let _lock = self.storage.acquire_lock()?;
        let backups = self.storage.list_backups_desc()?;
        let backup_id = select_backup_id(&backups, id)?;
        let mut state = self.storage.load_state()?;
        let channel = state.last_channel.unwrap_or_default();
        let from_version = state
            .current_version
            .clone()
            .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());
        let backup_dir = self.storage.backups_dir().join(&backup_id);
        let install_dir = installer::install_dir_from_current_exe().inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        installer::rollback(&install_dir, &backup_dir).inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        installer::health_check(&install_dir).inspect_err(|e| {
            let _ = record_rollback_failure_state(&self.storage, &mut state, &e.to_string());
        })?;
        let restored_version = read_installed_version(&install_dir.join(bin_name("gprj")));
        state.installed_at = Some(now_text());
        match restored_version {
            Ok(v) => {
                state.last_result = Some("rollback".to_string());
                state.last_error = None;
                state.current_version = Some(v.clone());
                self.storage.save_state(&state)?;
                Ok(UpdateResult {
                    channel,
                    from_version,
                    to_version: v,
                    backup_id: Some(backup_id),
                    updated: true,
                })
            }
            Err(e) => {
                // Rollback is already healthy; keep it successful but record parse warning.
                state.last_result = Some("rollback_version_unknown".to_string());
                state.last_error = Some(e.to_string());
                self.storage.save_state(&state)?;
                Ok(UpdateResult {
                    channel,
                    from_version,
                    to_version: "unknown".to_string(),
                    backup_id: Some(backup_id),
                    updated: true,
                })
            }
        }
    }
}

fn record_failure_state(
    storage: &SelfUpdateStorage,
    state: &mut SelfUpdateState,
    channel: ReleaseChannel,
    remote_version: Option<String>,
    result: &str,
    err: &str,
) -> RunResult<()> {
    state.last_checked_at = Some(now_text());
    state.last_channel = Some(channel);
    state.last_remote_version = remote_version;
    state.last_result = Some(result.to_string());
    state.last_error = Some(err.to_string());
    storage.save_state(state)
}

fn record_rollback_failure_state(
    storage: &SelfUpdateStorage,
    state: &mut SelfUpdateState,
    err: &str,
) -> RunResult<()> {
    state.last_result = Some("rollback_failed".to_string());
    state.last_error = Some(err.to_string());
    storage.save_state(state)
}

fn bin_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_string()
    }
}

fn temp_dir(prefix: &str) -> PathBuf {
    let mut p = std::env::temp_dir();
    p.push(format!("galaxy-self-update-{}-{}", prefix, now_id()));
    p
}

struct TempDirGuard {
    path: PathBuf,
}

impl TempDirGuard {
    fn new(prefix: &str) -> RunResult<Self> {
        let path = temp_dir(prefix);
        std::fs::create_dir_all(&path)
            .owe_res()
            .want("create self update temp directory")
            .with(("prefix", prefix))
            .with(("path", path.as_path()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDirGuard {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn select_backup_id(backups: &[String], id: Option<&str>) -> RunResult<String> {
    match id {
        Some(raw) => {
            if !is_valid_backup_id(raw) {
                return Err(RunReason::Args("invalid backup id".into())
                    .to_err()
                    .want("validate backup id")
                    .with(("backup_id", raw))
                    .with_detail(format!("backup_id={raw}, expected=14 digits")));
            }
            if backups.iter().any(|v| v == raw) {
                Ok(raw.to_string())
            } else {
                Err(RunReason::Args("backup id not found".into())
                    .to_err()
                    .want("select rollback backup id")
                    .with(("backup_id", raw))
                    .with_detail(format!("backup_id={raw}")))
            }
        }
        None => backups
            .first()
            .cloned()
            .ok_or_else(|| RunReason::Args("no backup found".into()).to_err())
            .want("select latest rollback backup"),
    }
}

fn is_valid_backup_id(input: &str) -> bool {
    input.len() == 14 && input.bytes().all(|b| b.is_ascii_digit())
}

fn now_id() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}

fn now_text() -> String {
    Utc::now().to_rfc3339()
}

fn manifest_base_url(channel: ReleaseChannel) -> &'static str {
    match channel {
        ReleaseChannel::Stable => MANIFEST_BASE_URL_STABLE,
        ReleaseChannel::Alpha => MANIFEST_BASE_URL_ALPHA,
        ReleaseChannel::Beta => MANIFEST_BASE_URL_BETA,
    }
}

fn read_installed_version(bin: &std::path::Path) -> RunResult<String> {
    let out = Command::new(bin)
        .arg("--version")
        .output()
        .owe_res()
        .want("run installed binary version command")
        .with(("bin", bin))?;
    if !out.status.success() {
        return Err(RunReason::Exec("version command failed".into())
            .to_err()
            .want("read installed binary version")
            .with(("bin", bin))
            .with_detail(format!("{} --version exit={}", bin.display(), out.status)));
    }
    let text = format!(
        "{} {}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    parse_version_from_text(&text).ok_or_else(|| {
        RunReason::Exec("cannot parse version output".into())
            .to_err()
            .want("parse installed binary version output")
            .with(("bin", bin))
            .with_detail(format!("bin={}", bin.display()))
    })
}

fn parse_version_from_text(text: &str) -> Option<String> {
    for tok in text.split_whitespace() {
        let normalized = tok
            .trim_matches(|c: char| ['"', ',', ':', ';', '(', ')'].contains(&c))
            .trim();
        if normalized.is_empty() {
            continue;
        }
        if let Ok(v) = installer::normalize_version(normalized) {
            return Some(v.to_string());
        }
    }
    None
}

#[allow(dead_code)]
fn json_text<T: Serialize>(v: &T) -> RunResult<String> {
    serde_json::to_string_pretty(v)
        .owe_data()
        .want("serialize json text")
}

#[allow(dead_code)]
fn _asset_for_target<'a>(
    assets: &'a std::collections::BTreeMap<String, ManifestAsset>,
    target: &str,
) -> RunResult<&'a ManifestAsset> {
    assets
        .get(target)
        .ok_or_else(|| RunReason::Exec("asset not found".into()).to_err())
}

#[cfg(test)]
mod tests {
    use super::{ReleaseChannel, manifest_base_url, parse_version_from_text, select_backup_id};

    #[test]
    fn manifest_base_url_matches_channel_branch() {
        assert_eq!(
            manifest_base_url(ReleaseChannel::Stable),
            "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/main/updates/stable"
        );
        assert_eq!(
            manifest_base_url(ReleaseChannel::Alpha),
            "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/alpha/updates/alpha"
        );
        assert_eq!(
            manifest_base_url(ReleaseChannel::Beta),
            "https://raw.githubusercontent.com/galaxy-sec/galaxy-flow/beta/updates/beta"
        );
    }

    #[test]
    fn backup_id_requires_membership() {
        let backups = vec!["20260305010101".to_string()];
        assert!(select_backup_id(&backups, Some("20260305010101")).is_ok());
        assert!(select_backup_id(&backups, Some("20260305010102")).is_err());
    }

    #[test]
    fn backup_id_rejects_path_like_input() {
        let backups = vec!["20260305010101".to_string()];
        assert!(select_backup_id(&backups, Some("../20260305010101")).is_err());
        assert!(select_backup_id(&backups, Some("/tmp/x")).is_err());
    }

    #[test]
    fn parse_version_from_output_text() {
        assert_eq!(
            parse_version_from_text("gprj 0.12.4"),
            Some("0.12.4".to_string())
        );
        assert_eq!(
            parse_version_from_text("gflow version v0.12.5-pre.1"),
            Some("0.12.5-pre.1".to_string())
        );
        assert_eq!(parse_version_from_text("version: unknown"), None);
    }
}
