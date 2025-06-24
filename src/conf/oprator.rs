use std::path::{Path, PathBuf};

use dirs::home_dir;
use orion_error::{ErrorConv, UvsSysFrom};
use orion_syspec::{tools::ensure_path, types::Tomlable};

use crate::{
    err::{RunError, RunResult},
    task_report::task_rc_config::{TaskCenterAPI, TASK_REPORT_CENTER},
};

use super::gxlconf::{GxlConf, ReportCenterConf};

// 加载任务配置,优先从环境变量中获取，如果没有则从默认路径中获取
pub fn load_gxl_config() {
    let galaxy_path = home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"));
    let task_config_path =
        std::env::var("CONF_PATH").unwrap_or(format!("{}/conf.toml", galaxy_path.display()));
    let path = Path::new(&task_config_path);
    if !path.exists() {
        warn!("conf.toml not found. Run in the default mode");
        return;
    } 
    let content = std::fs::read_to_string(path);
    match content {
        Ok(content) => {
            let res: Result<TaskCenterAPI, toml::de::Error> = toml::from_str(&content);
            match res {
                Ok(config) => {
                    let _ = TASK_REPORT_CENTER.set(config);
                }
                Err(e) => warn!("load conf.toml error: {}", e.message()),
            };
        }
        Err(e) => {
            warn!("load con.toml error: {}", e);
        }
    };
}

pub fn conf_path() -> Option<PathBuf> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        let conf_file = galaxy_root.join("conf.toml");
        if conf_file.exists() {
            return Some(conf_file);
        }
    }
    None
}

pub fn conf_init() -> RunResult<()> {
    if let Some(home_dir) = dirs::home_dir() {
        let galaxy_root = home_dir.join(".galaxy");
        ensure_path(&galaxy_root).err_conv()?;
        let conf_file = galaxy_root.join("conf.toml");
        let conf = GxlConf::new(ReportCenterConf::local(), true);
        conf.save_toml(&conf_file).err_conv()?;
        return Ok(());
    }
    Err(RunError::from_sys("get home dir failed!".to_string()))
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;
    use std::{
        env::{remove_var, set_var, temp_dir},
        fs::File,
        io::Write,
    };

    use crate::{conf::oprator::load_gxl_config, task_report::task_rc_config::TASK_REPORT_CENTER};

    // 加载任务配置测试
    #[tokio::test]
    async fn test_load_task_config() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let dir = temp_dir();
        let file_path = dir.join("conf.toml");
        if file_path.exists() {
            std::fs::remove_file(&file_path).assert();
        }

        let original_path = std::env::var("CONF_PATH");
        match original_path {
            Ok(_) => {
                load_gxl_config();
            }
            Err(_) => {
                // 写入测试内容
                let mut file = File::create(&file_path).assert();
                let config_content = r#"
                    report_enable = true
                    [report_svr]
                    domain = "127.0.0.1"
                    port = 8080
                    "#;
                writeln!(file, "{}", config_content).assert();

                // 临时修改路径指向我们的测试文件
                set_var("CONF_PATH", file_path.to_str().assert());

                load_gxl_config();
                remove_var("CONF_PATH");
            }
        }

        let task_result_config = TASK_REPORT_CENTER.get();
        assert!(task_result_config.is_some());
    }
}
