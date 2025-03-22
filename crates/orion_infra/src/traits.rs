use crate::types::AnyResult;

pub trait TomlStore {
    fn from_toml(path: &str) -> AnyResult<Self>
    where
        Self: Sized;
    fn save_toml(&self, path: &str, cover: bool) -> AnyResult<()>;
}
