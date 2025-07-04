use std::env;
use std::path::PathBuf;
use std::sync::Mutex;

static WORKDIR_LOCK: Mutex<()> = Mutex::new(());
pub struct WorkDir {
    original_dir: PathBuf,
}

impl WorkDir {
    pub fn change<S: Into<PathBuf>>(target_dir: S) -> std::io::Result<Self> {
        let original_dir = env::current_dir()?;
        let target = target_dir.into();
        info!("set current dir:{}", target.display());
        env::set_current_dir(&target)?;
        Ok(Self { original_dir })
    }
}

impl Drop for WorkDir {
    fn drop(&mut self) {
        info!("set current dir:{}", self.original_dir.display());
        if let Err(e) = env::set_current_dir(&self.original_dir) {
            log::error!("Failed to restore directory: {e}",);
        }
    }
}

pub struct WorkDirWithLock {
    original_dir: PathBuf,
    _lock: std::sync::MutexGuard<'static, ()>,
}

impl WorkDirWithLock {
    pub fn change<S: Into<PathBuf>>(target_dir: S) -> std::io::Result<Self> {
        let lock = WORKDIR_LOCK.lock().unwrap();
        let original_dir = env::current_dir()?;
        let target = target_dir.into();
        info!("set current dir:{}", target.display());
        env::set_current_dir(&target)?;
        Ok(Self {
            original_dir,
            _lock: lock,
        })
    }
}

impl Drop for WorkDirWithLock {
    fn drop(&mut self) {
        info!("set current dir:{}", self.original_dir.display());
        if let Err(e) = env::set_current_dir(&self.original_dir) {
            log::error!("Failed to restore directory: {e}",);
        }
    }
}
