use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env::home_dir;
use std::path::PathBuf;
use std::{fs, path::Path};
use toml::from_str;

lazy_static! {
    pub static ref TASK_REPORT_CENTER: OnceCell<TaskRCAPIConfig> = OnceCell::new();
}

// 任务结果配置
#[derive(Deserialize, Debug)]
pub struct TaskRCAPIConfig {
    pub task_callback_center: Option<HttpUrl>,
    pub task_reporting_center: Option<HttpUrl>,
    pub main_task_create_center: Option<HttpUrl>,
}

// 任务结果上报路径
#[derive(Deserialize, Clone, Debug)]
pub struct HttpUrl {
    pub url: String,
}

// 加载任务配置,优先从环境变量中获取，如果没有则从默认路径中获取
pub async fn load_task_config() {
    let galaxy_path = home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"));
    println!("load task config from: {}", galaxy_path.display());
    let task_config_path = std::env::var("TASK_RC_API_CONFIG_PATH")
        .unwrap_or(format!("{}/gflow_task_config.toml", galaxy_path.display()));
    let path = Path::new(&task_config_path);
    let content = fs::read_to_string(path);
    match content {
        Ok(content) => {
            let res: Result<TaskRCAPIConfig, toml::de::Error> = from_str(&content);
            match res {
                Ok(config) => {
                    println!("load task config success:{:#?}", config);
                    let _ = TASK_REPORT_CENTER.set(config);
                }
                Err(e) => info!("load task config error: {}", e.message()),
            };
        }
        Err(e) => {
            info!("load task_config toml error: {}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;
    use std::{
        env::{remove_var, set_var, temp_dir},
        fs::File,
        io::Write,
    };

    use crate::report_center::task_rc_config::TASK_REPORT_CENTER;

    use super::load_task_config;

    // 加载任务配置测试
    #[tokio::test]
    async fn test_load_task_config() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let dir = temp_dir();
        let file_path = dir.join("gflow_task_config.toml");
        if file_path.exists() {
            std::fs::remove_file(&file_path).assert();
        }

        let original_path = std::env::var("TASK_RC_API_CONFIG_PATH");
        match original_path {
            Ok(_) => {
                load_task_config().await;
            }
            Err(_) => {
                // 写入测试内容
                let mut file = File::create(&file_path).assert();
                let config_content = r#"[task_callback_center]
                    url = "http://127.0.0.1:8080/task/update_subtask_info/"

                    [task_reporting_center]
                    url = "http://127.0.0.1:8080/task/create_batch_subtask/"

                    [main_task_create_center]
                    url = "http://127.0.0.1:8080/task/create_main_task/"
                    "#;
                writeln!(file, "{}", config_content).assert();

                // 临时修改路径指向我们的测试文件
                set_var("TASK_RC_API_CONFIG_PATH", file_path.to_str().assert());

                load_task_config().await;
                remove_var("TASK_RC_API_CONFIG_PATH");
            }
        }

        let task_result_config = TASK_REPORT_CENTER.get().assert();

        // 验证全局变量
        assert!(task_result_config.main_task_create_center.is_some());
        assert!(task_result_config.task_callback_center.is_some());
        assert!(task_result_config.task_reporting_center.is_some());
    }
}
