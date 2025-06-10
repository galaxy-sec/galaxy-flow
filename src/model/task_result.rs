use crate::execution::VarSpace;
use crate::ExecReason;
use crate::{ability::prelude::ExecOut, execution::task::Task as ExecTask};
use once_cell::sync::OnceCell;
use serde::Serialize;
use std::env;
use std::sync::atomic::{AtomicU64, Ordering};
use std::{fs, path::Path};
use toml::from_str;

pub static TASK_ORDER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize, Clone)]
pub struct TaskResult {
    pub parent_id: String,
    pub name: String,       // 子任务名称
    pub log: String,        // 执行日志
    pub status: TaskStatus, // 执行状态
    pub order: u64,         // 执行顺序
}

/// Task status
#[derive(Debug, Clone, Serialize)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl TaskResult {
    pub fn from_task(task: ExecTask) -> TaskResult {
        TaskResult {
            parent_id: get_task_id(),
            name: task.name().clone(),
            log: String::new(),
            status: match task.result() {
                Ok(_) => TaskStatus::Completed,
                Err(_) => TaskStatus::Failed,
            },
            order: {
                TASK_ORDER.fetch_add(1, Ordering::SeqCst);
                TASK_ORDER.load(Ordering::SeqCst)
            }, //load 方法用于读取 TASK_ORDER 的当前值，Ordering::SeqCst 指定了一个内存顺序，确保读取操作是顺序一致的。
        }
    }

    pub fn from_result(
        task_name: String,
        result: &Result<(VarSpace, ExecOut), orion_error::StructError<ExecReason>>,
    ) -> TaskResult {
        let mut task_result = TaskResult {
            parent_id: get_task_id(),
            name: task_name,
            log: String::new(),
            status: TaskStatus::Pending,
            order: {
                TASK_ORDER.fetch_add(1, Ordering::SeqCst);
                TASK_ORDER.load(Ordering::SeqCst)
            },
        };
        match result {
            Ok((_, out)) => {
                if let ExecOut::Task(task) = out {
                    task_result.status = TaskStatus::Completed;
                    task_result.log = task.stdout.clone();
                }
            }
            Err(e) => {
                task_result.status = TaskStatus::Failed;
                task_result.log = e.to_string();
            }
        }
        task_result
    }
}

pub fn get_task_id() -> String {
    match env::var("task_id") {
        Ok(id) => id,
        Err(_) => "0".to_string(), // 如果没有设置 task_id，则返回 "0"
    }
}

use serde::Deserialize;

#[derive(Deserialize)]
pub struct TaskResultConfig {
    pub task_result_center: Option<TaskResultUrl>,
}

#[derive(Deserialize, Clone)]
pub struct TaskResultUrl {
    pub url: String,
}

lazy_static! {
    pub static ref TASK_RESULT_CONDIG: OnceCell<TaskResultConfig> = OnceCell::new();
}

pub fn load_task_config() {
    println!("load task config");
    let path = Path::new("src/config.toml");
    let content = fs::read_to_string(path);
    match content {
        Ok(content) => {
            let res: Result<TaskResultConfig, toml::de::Error> = from_str(&content);
            match res {
                Ok(config) => {
                    let _ = TASK_RESULT_CONDIG.set(config);
                }
                Err(e) => println!("load task config error: {}", e.message()),
            };
        }
        Err(e) => {
            println!("load toml error: {}", e)
        }
    };
}
