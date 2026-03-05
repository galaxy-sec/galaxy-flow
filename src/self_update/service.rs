use std::path::PathBuf;

use chrono::Utc;
use serde::Serialize;

use orion_error::ToStructError;

use crate::err::{RunReason, RunResult};

use super::client::SelfUpdateClient;
use super::installer;
use super::model::{
    AutoMode, CheckResult, ManifestAsset, ReleaseChannel, SelfUpdatePolicy, StatusResult,
    UpdateResult,
};
use super::storage::SelfUpdateStorage;

#[derive(Clone, Debug, Default)]
pub struct CheckRequest {
    pub channel: Option<ReleaseChannel>,
}

#[derive(Clone, Debug, Default)]
pub struct UpdateRequest {
    pub channel: Option<ReleaseChannel>,
    pub to_version: Option<String>,
    pub yes: bool,
    pub dry_run: bool,
    pub force: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AutoSetRequest {
    pub enabled: Option<bool>,
    pub mode: Option<AutoMode>,
    pub interval_hours: Option<u64>,
    pub channel: Option<ReleaseChannel>,
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
        let policy = self.storage.load_policy()?;
        let mut state = self.storage.load_state()?;
        state.current_version = Some(env!("CARGO_PKG_VERSION").to_string());
        Ok(StatusResult {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            install_dir,
            policy,
            state,
        })
    }

    pub async fn check(&self, req: CheckRequest) -> RunResult<CheckResult> {
        let mut state = self.storage.load_state()?;
        let policy = self.storage.load_policy()?;
        let channel = req.channel.unwrap_or(policy.channel);
        let temp_dir = temp_dir("check");
        std::fs::create_dir_all(&temp_dir).map_err(io_err)?;
        let manifest = self
            .client
            .fetch_manifest(&policy.manifest_base_url, channel, &temp_dir)
            .await
            .map_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    None,
                    "check_failed",
                    &e.to_string(),
                );
                e
            })?;

        let current = env!("CARGO_PKG_VERSION").to_string();
        let has_update = installer::is_remote_newer(&current, &manifest.version).map_err(|e| {
            let _ = record_failure_state(
                &self.storage,
                &mut state,
                channel,
                Some(manifest.version.clone()),
                "check_failed",
                &e.to_string(),
            );
            e
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
        let mut policy = self.storage.load_policy()?;
        if let Some(ch) = req.channel {
            policy.channel = ch;
        }
        let channel = policy.channel;
        let current = env!("CARGO_PKG_VERSION").to_string();

        let temp_dir = temp_dir("update");
        std::fs::create_dir_all(&temp_dir).map_err(io_err)?;

        let manifest = self
            .client
            .fetch_manifest(&policy.manifest_base_url, channel, &temp_dir)
            .await
            .map_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    None,
                    "update_failed",
                    &e.to_string(),
                );
                e
            })?;
        let remote = manifest.version.clone();
        if let Some(expect) = &req.to_version {
            if expect.trim() != remote {
                let err = RunReason::Args("target version mismatch".into())
                    .to_err()
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
        }

        let has_update = if req.force {
            true
        } else {
            installer::is_remote_newer(&current, &remote).map_err(|e| {
                let _ = record_failure_state(
                    &self.storage,
                    &mut state,
                    channel,
                    Some(remote.clone()),
                    "update_failed",
                    &e.to_string(),
                );
                e
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
        if !req.yes {
            let err = RunReason::Args("confirmation required".into())
                .to_err()
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

        let target = installer::detect_target_triple()?;
        let asset = manifest.assets.get(&target).ok_or_else(|| {
            RunReason::Exec("asset for target not found".into())
                .to_err()
                .with_detail(format!("target={target}"))
        })?;

        if req.dry_run {
            return Ok(UpdateResult {
                channel,
                from_version: current,
                to_version: remote,
                backup_id: None,
                updated: false,
            });
        }

        let asset_file = temp_dir.join(format!("update-{target}.tar.gz"));
        self.client
            .download_to_path(&asset.url, &asset_file)
            .await?;
        installer::verify_sha256(&asset_file, &asset.sha256)?;

        let unpack_dir = temp_dir.join("unpack");
        installer::extract_tar_gz(&asset_file, &unpack_dir)?;
        let new_gprj = installer::find_binary(&unpack_dir, bin_name("gprj").as_str())?;
        let new_gflow = installer::find_binary(&unpack_dir, bin_name("gflow").as_str())?;

        let install_dir = installer::install_dir_from_current_exe()?;
        let backup_id = now_id();
        let backup_dir = self.storage.create_backup_dir(&backup_id)?;

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
        let backup_id = if let Some(id) = id {
            id.to_string()
        } else {
            backups
                .first()
                .cloned()
                .ok_or_else(|| RunReason::Args("no backup found".into()).to_err())?
        };
        let backup_dir = self.storage.backups_dir().join(&backup_id);
        let install_dir = installer::install_dir_from_current_exe()?;
        installer::rollback(&install_dir, &backup_dir)?;
        installer::health_check(&install_dir)?;

        let mut state = self.storage.load_state()?;
        state.last_result = Some("rollback".to_string());
        state.last_error = None;
        state.installed_at = Some(now_text());
        self.storage.save_state(&state)?;

        Ok(UpdateResult {
            channel: state.last_channel.unwrap_or_default(),
            from_version: env!("CARGO_PKG_VERSION").to_string(),
            to_version: "rollback".to_string(),
            backup_id: Some(backup_id),
            updated: true,
        })
    }

    pub fn set_auto(&self, req: AutoSetRequest) -> RunResult<SelfUpdatePolicy> {
        let mut policy = self.storage.load_policy()?;
        if let Some(enabled) = req.enabled {
            policy.enabled = enabled;
        }
        if let Some(mode) = req.mode {
            policy.mode = mode;
        }
        if let Some(interval) = req.interval_hours {
            policy.interval_hours = interval.max(1);
        }
        if let Some(channel) = req.channel {
            policy.channel = channel;
        }
        self.storage.save_policy(&policy)?;
        Ok(policy)
    }
}

fn record_failure_state(
    storage: &SelfUpdateStorage,
    state: &mut super::model::SelfUpdateState,
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

fn now_id() -> String {
    Utc::now().format("%Y%m%d%H%M%S").to_string()
}

fn now_text() -> String {
    Utc::now().to_rfc3339()
}

fn io_err(err: std::io::Error) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

#[allow(dead_code)]
fn json_text<T: Serialize>(v: &T) -> RunResult<String> {
    serde_json::to_string_pretty(v).map_err(|e| RunReason::Exec(e.to_string()).to_err())
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
