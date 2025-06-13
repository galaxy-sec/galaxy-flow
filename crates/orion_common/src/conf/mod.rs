use anyhow::Context;
use serde::Serialize;
use std::path::Path;

// 确保目录存在，如果不存在则创建
pub fn ensure_directory_exists(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create path: {}", parent.to_string_lossy()))?;
    }
    Ok(())
}

// 将配置数据导出为 TOML 文件
pub fn export_toml<T>(conf: &T, file_name: &str) -> anyhow::Result<()>
where
    T: Serialize,
{
    // 将配置数据序列化为 TOML 字符串
    let toml = toml::to_string(conf)?;
    let path = Path::new(file_name);

    // 确保目录存在
    ensure_directory_exists(path)?;

    // 将 TOML 字符串写入文件
    std::fs::write(path, toml).with_context(|| format!("write toml file : {}", file_name))?;

    Ok(())
}
