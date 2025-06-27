use crate::execution::task::Task;
use crate::task_report::main_task::get_task_parent_id;
use crate::task_report::task_notification::TaskNotice;
use serde::Serialize;

// 返回至任务报告中心任务执行结果
#[derive(Debug, Serialize, Clone)]
pub struct TaskReport {
    pub parent_id: i64,
    pub name: String,          // 子任务名称
    pub log: String,           // 执行日志
    pub status: SubTaskStatus, // 执行状态
    pub order: u32,            // 执行顺序
}

/// 任务状态
#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum SubTaskStatus {
    Pending,
    Inprogress,
    Success,
    Failure,
}

impl TaskReport {
    // 转化报告中心的返回结果
    pub fn from_task_and_notice(task: Task, task_notice: TaskNotice) -> TaskReport {
        let mut running_log = task.stdout().clone();
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
                Err(_) => SubTaskStatus::Failure,
            },
            order: task_notice.order,
        }
    }
}
