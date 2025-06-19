use crate::execution::task::Task as ExecTask;
use crate::util::http_handle::{get_create_maintask_url, send_http_request};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::env;
use std::sync::Mutex;
use std::{fs, path::Path};
use time::{format_description, OffsetDateTime};
use toml::from_str;

lazy_static::lazy_static! {
    static ref NEXT_ORDER: Mutex<u16> = Mutex::new(0);
}

// 任务执行结果
#[derive(Debug, Serialize, Clone)]
pub struct TaskCallBackResult {
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

impl TaskCallBackResult {
    // 转化成任务中心的返回结果
    pub fn from_task_with_order(task: ExecTask, taskbody: TaskBody) -> TaskCallBackResult {
        let mut running_log = String::new();
        for action in task.actions() {
            let stdout = action.stdout();
            if !stdout.is_empty() {
                running_log.push_str(&format!("{}\n", stdout));
            }
        }
        TaskCallBackResult {
            parent_id: get_task_parent_id().unwrap().parse::<i64>().unwrap_or(0),
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
#[derive(Deserialize)]
pub struct TaskResultConfig {
    pub task_callback_center: Option<HttpUrl>,
    pub task_reporting_center: Option<HttpUrl>,
    pub create_maintask_url: Option<HttpUrl>,
}

// 任务结果上报路径
#[derive(Deserialize, Clone)]
pub struct HttpUrl {
    pub url: String,
}

// 批量任务上报结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct BatchTaskRequest {
    pub tasks: Vec<TaskBody>,
}

// 子任务结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaskBody {
    pub parent_id: i64,
    pub name: String,        // 子任务名称
    pub description: String, // 子任务描述
    pub order: u16,          // 执行顺序
}

impl TaskBody {
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
    pub fn new() -> TaskBody {
        TaskBody {
            parent_id: get_task_parent_id().unwrap().parse::<i64>().unwrap_or(0),
            name: String::new(),
            description: String::new(),
            order: 0,
        }
    }
}

lazy_static! {
    pub static ref TASK_RESULT_CONDIG: OnceCell<TaskResultConfig> = OnceCell::new();
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
    let task_config_path = std::env::var("TASK_CONFIG_PATH")
        .unwrap_or("/usr/local/bin/gflow_task_config.toml".to_string());
    let path = Path::new(&task_config_path);
    let content = fs::read_to_string(path);
    match content {
        Ok(content) => {
            let res: Result<TaskResultConfig, toml::de::Error> = from_str(&content);
            match res {
                Ok(config) => {
                    let _ = TASK_RESULT_CONDIG.set(config);
                }
                Err(e) => info!("load task config error: {}", e.message()),
            };
        }
        Err(e) => {
            info!("load task_config toml error: {}", e)
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
    let now = datetime.format(&format.unwrap()).unwrap();
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
    if let Some(url) = get_create_maintask_url() {
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
    use orion_error::TestAssertWithMsg;
    use std::{env::{temp_dir, remove_var, set_var}, fs::File, io::Write};

    use crate::task_callback_result::{load_task_config, TASK_RESULT_CONDIG};

    // 加载任务配置测试
    #[tokio::test]
    async fn test_load_secfile_with_values() {
        // 创建临时目录和文件
        //let dir = PathBuf::from("./temp");
        let dir = temp_dir();
        let file_path = dir.join("gflow_task_config.toml");
        if file_path.exists() {
            std::fs::remove_file(&file_path).assert("remove file");
        }

        let original_path = std::env::var("TASK_CONFIG_PATH");
        match original_path {
            Ok(_) => {
                load_task_config().await;
            }
            Err(_) => {
                 // 写入测试内容
                let mut file = File::create(&file_path).unwrap();
                let config_content = r#"[task_callback_center]
                    url = "http://127.0.0.1:8080/task/update_subtask_info/"

                    [task_reporting_center]
                    url = "http://127.0.0.1:8080/task/create_batch_subtask/"

                    [create_maintask_url]
                    url = "http://127.0.0.1:8080/task/create_main_task/"
                    "#;
                writeln!(file, "{}", config_content).unwrap();
            

                // 临时修改路径指向我们的测试文件
                set_var("TASK_CONFIG_PATH", file_path.to_str().unwrap());

                load_task_config().await;
                remove_var("TASK_CONFIG_PATH");
            }
        }
       

        let task_result_config = TASK_RESULT_CONDIG.get().unwrap();

        // 验证全局变量
        assert!(task_result_config.create_maintask_url.is_some());
        assert!(task_result_config.task_callback_center.is_some());
        assert!(task_result_config.task_reporting_center.is_some());
    }
}
