use serde_yaml;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use crate::ai::{AiErrReason, AiResult};

use super::structures::{AiConfig, FileConfig};
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

    /// 加载配置文件
    pub fn load_file_config(&self) -> AiResult<FileConfig> {
        let config_path = Self::ensure_config_dir()?.join("ai.yml");
        self.load_config_from_path(&config_path)
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

    /// 加载完整配置（文件 + 环境变量）
    pub fn load_config() -> AiResult<AiConfig> {
        let loader = Self::new();

        // 首先从环境变量加载基础配置
        let mut config = AiConfig::from_env();

        // 尝试加载配置文件
        match loader.load_file_config() {
            Ok(file_config) => {
                println!("📄 File config loaded successfully, merging...");
                // 合并配置文件内容
                loader.merge_file_config(&mut config, file_config)?;
                println!("✅ File config merged successfully");
            }
            Err(e) => {
                // 配置文件不存在时的优雅降级
                log::info!(
                    "Config file not found or invalid, using environment variables only: {e}",
                );
            }
        }

        Ok(config)
    }

    /// 合并文件配置到主配置
    pub fn merge_file_config(
        &self,
        config: &mut AiConfig,
        file_config: FileConfig,
    ) -> AiResult<()> {
        config.file_config = Some(file_config.clone());

        if !file_config.enabled {
            return Ok(());
        }

        // 如果启用文件配置且要覆盖环境变量，则更新provider配置
        if file_config.override_env {
            // 这里后续可以添加从文件中读取provider配置的逻辑
            // 目前保持向后兼容，以环境变量为准
            log::info!("File config enabled, would merge provider settings");
        }

        Ok(())
    }
}
