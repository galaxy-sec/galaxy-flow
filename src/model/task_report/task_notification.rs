use crate::task_report::main_task::get_task_parent_id;
use serde::Deserialize;
use serde::Serialize;
use time::OffsetDateTime;

// 批量任务上报结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaskOutline {
    pub tasks: Vec<TaskNotice>,
}

// 子任务结构体
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct TaskNotice {
    pub parent_id: String,
    pub name: String,        // 子任务名称s
    pub description: String, // 子任务描述
    pub order: u32,          // 执行顺序
}

impl TaskNotice {
    pub fn new() -> TaskNotice {
        let parent_id = get_task_parent_id().unwrap_or_default();
        let order = OffsetDateTime::now_utc();
        TaskNotice {
            parent_id,
            name: String::new(),
            description: String::new(),
            order: order.nanosecond(),
        }
    }
}
