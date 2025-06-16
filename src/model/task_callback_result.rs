use crate::execution::task::Task as ExecTask;
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::env;
use std::sync::Mutex;
use std::{fs, path::Path};
use toml::from_str;

lazy_static::lazy_static! {
    static ref NEXT_ORDER: Mutex<u16> = Mutex::new(0);
}

// 任务执行结果
#[derive(Debug, Serialize, Clone)]
pub struct TaskCallBackResult {
    pub parent_id: i64,
    pub name: String,       // 子任务名称
    pub log: String,        // 执行日志
    pub status: TaskStatus, // 执行状态
    pub order: u16,         // 执行顺序
}

/// 任务状态
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    Completed,
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
            parent_id: get_task_parent_id().parse::<i64>().unwrap_or(0),
            name: task.name().clone(),
            log: running_log,
            status: match task.result() {
                Ok(_) => TaskStatus::Completed,
                Err(_) => TaskStatus::Failed,
            },
            order: taskbody.order,
        }
    }
}

// 获取当前任务的父id
pub fn get_task_parent_id() -> String {
    match env::var("task_id") {
        Ok(id) => id,
        Err(_) => "0".to_string(), // 如果没有设置 task_id，则返回 "0"
    }
}

use serde::Deserialize;

// 任务结果配置
#[derive(Deserialize)]
pub struct TaskResultConfig {
    pub task_callback_center: Option<TaskResultUrl>,
    pub task_reporting_center: Option<TaskResultUrl>,
}

// 任务结果上报路径
#[derive(Deserialize, Clone)]
pub struct TaskResultUrl {
    pub url: String,
}

// 子任务上报中心路径
#[derive(Deserialize, Clone)]
pub struct TaskReportCenterUrl {
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
            parent_id: get_task_parent_id().parse::<i64>().unwrap_or(0),
            name: String::new(),
            description: String::new(),
            order: 0,
        }
    }
}

lazy_static! {
    pub static ref TASK_RESULT_CONDIG: OnceCell<TaskResultConfig> = OnceCell::new();
}

pub fn load_task_config() {
    let path = Path::new("./_gal/task_config.toml");
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
