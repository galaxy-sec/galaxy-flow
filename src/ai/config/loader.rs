use serde_yaml;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ai::{AiErrReason, AiResult};

use super::structures::FileConfig;
use orion_error::ToStructError;

use orion_error::UvsConfFrom;
/// 配置加载器，支持文件加载和变量替换
#[derive(Default)]
pub struct ConfigLoader {
    // 配置加载器状态
}

impl ConfigLoader {
    /// 创建新的配置加载器
    pub fn new() -> Self {
        Self::default()
    }

    /// 确保配置目录存在
    pub fn ensure_config_dir() -> AiResult<PathBuf> {
        let config_dir = dirs::home_dir()
            .ok_or_else(|| AiErrReason::ConfigError("Home directory not found".to_string()))?
            .join(".galaxy");

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                AiErrReason::ConfigError(format!("Failed to create config directory: {e}"))
            })?;
        }

        Ok(config_dir)
    }

    /// 从指定路径加载配置文件
    pub fn load_config_from_path(&self, config_path: &Path) -> AiResult<FileConfig> {
        if !config_path.exists() {
            return AiErrReason::from_conf(format!(
                "Config file not found: {}",
                config_path.display()
            ))
            .err_result();
        }

        let content = fs::read_to_string(config_path).map_err(|e| {
            AiErrReason::ConfigError(format!(
                "Failed to read config file {}: {}",
                config_path.display(),
                e
            ))
        })?;

        let mut file_config: FileConfig = serde_yaml::from_str(&content).map_err(|e| {
            AiErrReason::ConfigError(format!("Invalid YAML in {}: {}", config_path.display(), e))
        })?;

        file_config.config_path = config_path.to_path_buf();

        Ok(file_config)
    }
}
