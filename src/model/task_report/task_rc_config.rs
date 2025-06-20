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
    pub report_enable: bool,
    pub report_svr: ReportSVR,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ReportSVR {
    pub domain: String,
    pub port: u16,
    // 任务回调中心
    pub task_notice_center: String,
    pub task_report_center: String,
    pub main_task_create_center: String,
}
// 加载任务配置,优先从环境变量中获取，如果没有则从默认路径中获取
pub async fn load_task_config() {
    let galaxy_path = home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"));
    let task_config_path = std::env::var("TASK_RC_API_CONFIG_PATH")
        .unwrap_or(format!("{}/conf.toml", galaxy_path.display()));
    let path = Path::new(&task_config_path);
    let content = fs::read_to_string(path);
    match content {
        Ok(content) => {
            let res: Result<TaskRCAPIConfig, toml::de::Error> = from_str(&content);
            match res {
                Ok(config) => {
                    let _ = TASK_REPORT_CENTER.set(config);
                }
                Err(e) => println!("load task config error: {}", e.message()),
            };
        }
        Err(e) => {
            println!("load task_config toml error: {}", e);
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

    use crate::task_report::task_rc_config::TASK_REPORT_CENTER;

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
                let config_content = r#"
                    report_enable = true
                    [report_svr]
                    domain = "127.0.0.1"
                    port = 8080
                    task_notice_center = "/task/create_batch_subtask/"
                    task_report_center = "/task/update_subtask_info/"
                    main_task_create_center = "/task/create_main_task/"
                    "#;
                writeln!(file, "{}", config_content).assert();

                // 临时修改路径指向我们的测试文件
                set_var("TASK_RC_API_CONFIG_PATH", file_path.to_str().assert());

                load_task_config().await;
                remove_var("TASK_RC_API_CONFIG_PATH");
            }
        }

        let task_result_config = TASK_REPORT_CENTER.get();
        assert!(task_result_config.is_some());
    }
}
