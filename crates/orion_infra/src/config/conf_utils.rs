use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use crate::types::AnyResult;
use anyhow::{anyhow, Context};
use serde::Serialize;

pub fn save_toml_config<T: serde::Serialize>(obj: &T, path: &str, cover: bool) -> AnyResult<()> {
    handle_exists_file(path, cover)?;
    export_to_toml(&obj, path)?;
    println!("success init : {} ", path);
    Ok(())
}

fn handle_exists_file(path: &str, cover: bool) -> AnyResult<()> {
    if std::path::Path::new(path).exists() {
        if cover {
            std::fs::remove_file(path)?;
        } else {
            return Err(anyhow!("{} already exists", path));
        }
    }
    Ok(())
}

pub fn save_data(data: &str, path_str: &str, cover: bool) -> AnyResult<()> {
    handle_exists_file(path_str, cover)?;
    let path = Path::new(path_str);
    if let Some(value) = path.parent() {
        std::fs::create_dir_all(value)?;
    }
    let mut file = std::fs::File::create(path)?;
    file.write_all(data.as_bytes())?;
    println!("success init : {} ", path_str);
    Ok(())
}

pub fn backup_clean(path: &str) -> AnyResult<()> {
    if std::path::Path::new(path).exists() {
        std::fs::copy(path, format!("{}.bak", path))?;
        std::fs::remove_file(path)?;
    }
    Ok(())
}

pub fn clear_file(path: &str) {
    if std::path::Path::new(path).exists() {
        std::fs::remove_file(path).unwrap_or_else(|_| panic!("clean {} failed!", path));
    }
}

pub fn export_to_toml<T>(conf: &T, file_name: &str) -> AnyResult<()>
where
    T: Serialize,
{
    let toml = toml::to_string(conf)?;
    let path = Path::new(file_name);
    if let Some(value) = path.parent() {
        std::fs::create_dir_all(value)
            .with_context(|| format!("crate path: {}", value.to_string_lossy()))?;
    }
    std::fs::write(path, toml).with_context(|| format!("write toml file : {}", file_name))?;
    Ok(())
}

pub fn import_from_toml<T: serde::de::DeserializeOwned>(path: &str) -> AnyResult<T> {
    let mut f = File::open(path).with_context(|| format!("conf file not found: {}", path))?;
    let mut buffer = Vec::with_capacity(10240);
    f.read_to_end(&mut buffer)
        .with_context(|| format!("conf file: {}", path))?;
    let conf_data = String::from_utf8(buffer)?;
    let conf: T = toml::from_str(conf_data.as_str())?;
    Ok(conf)
}

pub fn read_file(path: &str) -> AnyResult<String> {
    let mut f = File::open(path).with_context(|| format!("conf file not found: {}", path))?;
    let mut buffer = Vec::with_capacity(10240);
    f.read_to_end(&mut buffer)
        .with_context(|| format!("conf file: {}", path))?;
    Ok(String::from_utf8(buffer)?)
}
