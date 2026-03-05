use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::PathBuf;

use orion_error::ToStructError;

use crate::err::{RunReason, RunResult};

use super::model::{SelfUpdatePolicy, SelfUpdateState};

const GALAXY_DIR: &str = ".galaxy";
const SELF_UPDATE_DIR: &str = "self_update";
const POLICY_FILE: &str = "policy.toml";
const STATE_FILE: &str = "state.json";
const LOCK_FILE: &str = "lock";
const BACKUPS_DIR: &str = "backups";

pub struct FileLock {
    path: PathBuf,
}

impl Drop for FileLock {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

#[derive(Clone, Debug)]
pub struct SelfUpdateStorage {
    root: PathBuf,
}

impl SelfUpdateStorage {
    pub fn new() -> RunResult<Self> {
        let home = dirs::home_dir().ok_or_else(|| {
            RunReason::Args("cannot resolve home directory".into())
                .to_err()
                .with_detail("self update needs home dir")
        })?;
        let root = home.join(GALAXY_DIR).join(SELF_UPDATE_DIR);
        let this = Self { root };
        this.ensure_layout()?;
        Ok(this)
    }

    pub fn policy_path(&self) -> PathBuf {
        self.root.join(POLICY_FILE)
    }

    pub fn state_path(&self) -> PathBuf {
        self.root.join(STATE_FILE)
    }

    pub fn lock_path(&self) -> PathBuf {
        self.root.join(LOCK_FILE)
    }

    pub fn backups_dir(&self) -> PathBuf {
        self.root.join(BACKUPS_DIR)
    }

    pub fn ensure_layout(&self) -> RunResult<()> {
        fs::create_dir_all(self.backups_dir()).map_err(io_err)?;
        Ok(())
    }

    pub fn load_policy(&self) -> RunResult<SelfUpdatePolicy> {
        let path = self.policy_path();
        if !path.exists() {
            let def = SelfUpdatePolicy::default();
            self.save_policy(&def)?;
            return Ok(def);
        }
        let content = fs::read_to_string(&path).map_err(io_err)?;
        toml::from_str::<SelfUpdatePolicy>(&content).map_err(parse_err)
    }

    pub fn save_policy(&self, policy: &SelfUpdatePolicy) -> RunResult<()> {
        let path = self.policy_path();
        let content = toml::to_string_pretty(policy).map_err(parse_err)?;
        fs::write(path, content).map_err(io_err)?;
        Ok(())
    }

    pub fn load_state(&self) -> RunResult<SelfUpdateState> {
        let path = self.state_path();
        if !path.exists() {
            return Ok(SelfUpdateState::default());
        }
        let content = fs::read_to_string(path).map_err(io_err)?;
        serde_json::from_str::<SelfUpdateState>(&content).map_err(parse_err)
    }

    pub fn save_state(&self, state: &SelfUpdateState) -> RunResult<()> {
        let path = self.state_path();
        let content = serde_json::to_string_pretty(state).map_err(parse_err)?;
        fs::write(path, content).map_err(io_err)?;
        Ok(())
    }

    pub fn acquire_lock(&self) -> RunResult<FileLock> {
        let path = self.lock_path();
        let mut file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(io_err)?;
        file.write_all(b"self-update-lock").map_err(io_err)?;
        Ok(FileLock { path })
    }

    pub fn create_backup_dir(&self, backup_id: &str) -> RunResult<PathBuf> {
        let dir = self.backups_dir().join(backup_id);
        fs::create_dir_all(&dir).map_err(io_err)?;
        Ok(dir)
    }

    pub fn list_backups_desc(&self) -> RunResult<Vec<String>> {
        let mut list = Vec::new();
        for item in fs::read_dir(self.backups_dir()).map_err(io_err)? {
            let item = item.map_err(io_err)?;
            if item.file_type().map_err(io_err)?.is_dir() {
                let file_name = item.file_name();
                if let Some(name) = file_name.to_str() {
                    list.push(name.to_string());
                }
            }
        }
        list.sort();
        list.reverse();
        Ok(list)
    }
}

fn io_err(err: io::Error) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}

fn parse_err(err: impl std::fmt::Display) -> crate::err::RunError {
    RunReason::Exec(err.to_string()).to_err()
}
