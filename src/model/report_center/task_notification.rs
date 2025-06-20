use crate::report_center::main_task::get_task_parent_id;
use serde::Deserialize;
use serde::Serialize;
use std::sync::Mutex;

// 任务执行顺序
lazy_static::lazy_static! {
    static ref NEXT_ORDER: Mutex<u16> = Mutex::new(0);
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
    pub fn new() -> TaskNotice {
        let parent_id = get_task_parent_id().unwrap_or_default();
        TaskNotice {
            parent_id: parent_id.parse::<i64>().unwrap_or(0),
            name: String::new(),
            description: String::new(),
            order: 0,
        }
    }

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
}



#[cfg(test)]
mod tests {
    use crate::execution::task::Task as FlowTask;
    use crate::report_center::task_notification::TaskNotice;
    use crate::report_center::task_report::TaskReport;

    #[test]
    fn test_set_order() {
        let mut flow_task1 = FlowTask::from("test");
        let mut task_notice1 = TaskNotice {
            parent_id: 1,
            name: String::from("Notice Task1"),
            description: String::from("Notice Description"),
            order: 0,
        };
        task_notice1.set_order(); // Set order for the task notice
        flow_task1.finish(); // Simulate finishing the task
        let task_report1 = TaskReport::from_flowtask_and_notice(flow_task1, task_notice1);

        let mut flow_task2 = FlowTask::from("test");
        let mut task_notice2 = TaskNotice {
            parent_id: 1,
            name: String::from("Notice Task2"),
            description: String::from("Notice Description"),
            order: 0,
        };
        task_notice2.set_order(); // Set order for the task notice
        flow_task2.finish(); // Simulate finishing the task
        let task_report2 = TaskReport::from_flowtask_and_notice(flow_task2, task_notice2);
        
        assert_eq!(task_report1.order, 1);
        assert_eq!(task_report2.order, 2);
    }
}


