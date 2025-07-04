use log::warn;

/// 配置标准操作接口
pub trait ConfigLifecycle {
    /// 尝试加载配置文件（安全方法）
    fn try_load(path: &str) -> Option<Self>
    where
        Self: Sized,
    {
        if std::path::Path::new(path).exists() {
            match Self::load(path) {
                Ok(conf) => Some(conf),
                Err(e) => {
                    warn!("load conf error: {e}");
                    None
                }
            }
        } else {
            None
        }
    }

    /// 强制加载配置文件
    fn load(path: &str) -> ConfResult<Self>
    where
        Self: Sized;

    /// 初始化配置文件
    fn init(&self, path: &str) -> ConfResult<()>
    where
        Self: Sized;

    /// 安全清理配置文件
    fn safe_clean(path: &str) -> ConfResult<()>;
    fn save(&self, path: &str) -> ConfResult<()>;
}

#[derive(Debug, thiserror::Error)]
pub enum ConfError {
    #[error("配置文件加载失败: {0}")]
    LoadError(String),
    #[error("配置文件保存失败: {0}")]
    SaveError(String),
    #[error("not exists: {0}")]
    NotExists(String),
}

pub type ConfResult<T> = Result<T, ConfError>;

impl ConfError {
    pub fn from_load(e: anyhow::Error) -> Self {
        Self::LoadError(e.to_string())
    }
    pub fn from_save(e: anyhow::Error) -> Self {
        Self::SaveError(e.to_string())
    }
    pub fn not_exists(e: anyhow::Error) -> Self {
        Self::NotExists(e.to_string())
    }
}
