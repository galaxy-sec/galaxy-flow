use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use orion_error::ToStructError;

use crate::err::{RunReason, RunResult};

use super::model::{SelfUpdatePolicy, SelfUpdateState};

const GALAXY_DIR: &str = ".galaxy";
const SELF_UPDATE_DIR: &str = "self_update";
const POLICY_FILE: &str = "policy.toml";
const STATE_FILE: &str = "state.json";
const LOCK_FILE: &str = "lock";
const BACKUPS_DIR: &str = "backups";
const STALE_LOCK_MAX_AGE_SECS: u64 = 24 * 60 * 60;

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
        match create_lock_file(&path) {
            Ok(lock) => Ok(lock),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => {
                if lock_is_stale(&path, Duration::from_secs(STALE_LOCK_MAX_AGE_SECS)) {
                    if let Some(pid) = read_lock_pid(&path) {
                        if process_is_running(pid) {
                            return Err(RunReason::Exec("self update is busy".into())
                                .to_err()
                                .with_detail(format!(
                                    "lock_file={}, pid={} still running",
                                    path.display(),
                                    pid
                                )));
                        }
                    }
                    let _ = fs::remove_file(&path);
                    create_lock_file(&path).map_err(io_err)
                } else {
                    Err(RunReason::Exec("self update is busy".into())
                        .to_err()
                        .with_detail(format!(
                            "lock_file={}, remove it manually if previous process crashed",
                            path.display()
                        )))
                }
            }
            Err(err) => Err(io_err(err)),
        }
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

fn create_lock_file(path: &Path) -> io::Result<FileLock> {
    let mut file = OpenOptions::new().create_new(true).write(true).open(path)?;
    file.write_all(format!("pid={}\n", std::process::id()).as_bytes())?;
    Ok(FileLock {
        path: path.to_path_buf(),
    })
}

fn lock_is_stale(path: &Path, max_age: Duration) -> bool {
    let Ok(meta) = fs::metadata(path) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    let Ok(elapsed) = modified.elapsed() else {
        return false;
    };
    elapsed > max_age
}

fn read_lock_pid(path: &Path) -> Option<u32> {
    let text = fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if let Some(v) = line.strip_prefix("pid=") {
            if let Ok(pid) = v.trim().parse::<u32>() {
                return Some(pid);
            }
        }
    }
    None
}

#[cfg(unix)]
fn process_is_running(pid: u32) -> bool {
    if pid == 0 {
        return false;
    }
    let rc = unsafe { libc::kill(pid as i32, 0) };
    if rc == 0 {
        return true;
    }
    std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

#[cfg(not(unix))]
fn process_is_running(_pid: u32) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use super::read_lock_pid;

    #[test]
    fn parse_lock_pid() {
        let dir = tempfile::tempdir().expect("tmpdir");
        let path = dir.path().join("lock");
        std::fs::write(&path, "pid=12345\n").expect("write lock");
        assert_eq!(read_lock_pid(&path), Some(12345));
    }
}
