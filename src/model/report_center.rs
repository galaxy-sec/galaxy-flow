use crate::execution::task::Task as FlowTask;
use crate::util::http_handle::{get_main_task_create_url, send_http_request};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::env::{self, home_dir};
use std::path::PathBuf;
use std::sync::Mutex;
use std::{fs, path::Path};
use time::{format_description, OffsetDateTime};
use toml::from_str;

lazy_static::lazy_static! {
    static ref NEXT_ORDER: Mutex<u16> = Mutex::new(0);
}

// 返回至任务报告中心任务执行结果
#[derive(Debug, Serialize, Clone)]
pub struct TaskReport {
    pub parent_id: i64,
    pub name: String,          // 子任务名称
    pub log: String,           // 执行日志
    pub status: SubTaskStatus, // 执行状态
    pub order: u16,            // 执行顺序
}

/// 任务状态
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum SubTaskStatus {
    Created,
    Running,
    Success,
    Failed,
}

impl TaskReport {
    // 转化成任务中心的返回结果
    pub fn from_task_with_order(task: FlowTask, taskbody: TaskNotice) -> TaskReport {
        let mut running_log = String::new();
        for action in task.actions() {
            let stdout = action.stdout();
            if !stdout.is_empty() {
                running_log.push_str(&format!("{}\n", stdout));
            }
        }
        TaskReport {
            parent_id: get_task_parent_id()
                .unwrap_or_default()
                .parse::<i64>()
                .unwrap_or(0),
            name: task.name().clone(),
            log: running_log,
            status: match task.result() {
                Ok(_) => SubTaskStatus::Success,
                Err(_) => SubTaskStatus::Failed,
            },
            order: taskbody.order,
        }
    }
}

// 获取当前任务的父id
pub fn get_task_parent_id() -> Option<String> {
    env::var("task_id").ok()
}

use serde::Deserialize;

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

// 批量任务上报结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaskRecord {
    pub tasks: Vec<TaskNotice>,
}

// 子任务结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaskNotice {
    pub parent_id: i64,
    pub name: String,        // 子任务名称
    pub description: String, // 子任务描述
    pub order: u16,          // 执行顺序
}

impl TaskNotice {
    // 为任务设置执行顺序
    pub fn set_order(&mut self) {
        let next_order = match NEXT_ORDER.lock() {
            // 如果锁获取成功，则返回下一个order
            Ok(mut next_order) => {
                *next_order += 1;
                *next_order
            }
            // 如果锁获取失败，则返回0
            Err(_) => {
                println!("next_order lock error");
                0
            }
        };
        self.order = next_order;
    }
    pub fn new() -> TaskNotice {
        let parent_id = get_task_parent_id().unwrap_or_default();
        TaskNotice {
            parent_id: parent_id.parse::<i64>().unwrap_or(0),
            name: String::new(),
            description: String::new(),
            order: 0,
        }
    }
}

lazy_static! {
    pub static ref TASK_REPORT_CENTER: OnceCell<TaskRCAPIConfig> = OnceCell::new();
}

#[derive(Debug, Serialize, Clone)]
pub struct MainTask {
    pub maintask_name: String,
    pub worker_name: String,
    pub description: Option<String>,
    pub task_type: String,
    pub id: i64,
}
// 加载任务配置,优先从环境变量中获取，如果没有则从默认路径中获取
pub async fn load_task_config() {
    let galaxy_path = home_dir()
        .map(|x| x.join(".galaxy"))
        .unwrap_or(PathBuf::from("./"));
    println!("load task config from: {}", galaxy_path.display());
    let task_config_path = std::env::var("TASK_RC_API_CONFIG_PATH").unwrap_or(format!(
        "{}/gflow_task_config.toml",
        galaxy_path.display()
    ));
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

pub async fn create_main_task(task_name: String) {
    // 创建主任务
    let datetime = OffsetDateTime::now_utc();
    let format: Result<
        Vec<format_description::BorrowedFormatItem<'_>>,
        time::error::InvalidFormatDescription,
    > = format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second]");
    let mut now = String::new();
    match format {
        Ok(fmt) => now = datetime.format(&fmt).unwrap_or_default(),
        Err(e) => println!("create main task time format error: {}", e),
    }
    let parent_id = datetime.unix_timestamp();
    let main_task = MainTask {
        id: parent_id,
        maintask_name: format!("{} {}", task_name, now),
        worker_name: String::new(),
        description: Some(task_name.clone()),
        task_type: task_name,
    };
    // 设置环境变量中的父id
    std::env::set_var("task_id", parent_id.to_string());
    // 创建主任务
    if let Some(url) = get_main_task_create_url() {
        match send_http_request(main_task, &url).await {
            Ok(response) => {
                if response.status().is_success() {
                    println!("create maintask success");
                } else {
                    println!("create maintask error: {:?}", response.text().await);
                }
            }
            Err(e) => {
                println!("create maintask error: {}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use orion_error::TestAssert;
    use std::{
        env::{remove_var, set_var, temp_dir},
        fs::File,
        io::Write,
    };

    use crate::report_center::{load_task_config, TASK_REPORT_CENTER};

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
