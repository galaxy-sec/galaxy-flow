use crate::{config::conf_utils::backup_clean, traits::TomlStore, types::AnyResult};

use super::{
    conf_utils::{import_from_toml, save_toml_config},
    traits::{ConfError, ConfResult},
    ConfigLifecycle,
};

impl<T> TomlStore for T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn from_toml(path: &str) -> AnyResult<Self> {
        import_from_toml::<T>(path)
    }
    fn save_toml(&self, path: &str, cover: bool) -> AnyResult<()> {
        save_toml_config(self, path, cover)
        //self.save_toml(path, cover)
    }
}

impl<T: TomlStore> ConfigLifecycle for T
where
    T: serde::Serialize + serde::de::DeserializeOwned,
{
    fn load(path: &str) -> ConfResult<Self>
    where
        Self: Sized,
    {
        Self::from_toml(path).map_err(ConfError::from_load)
    }

    fn init(&self, path: &str) -> ConfResult<()>
    where
        Self: Sized,
    {
        Self::safe_clean(path)?;
        self.save(path)
    }

    fn safe_clean(path: &str) -> ConfResult<()> {
        backup_clean(path).map_err(ConfError::not_exists)
    }

    fn save(&self, path: &str) -> ConfResult<()> {
        self.save_toml(path, true).map_err(ConfError::from_save)
    }
}
