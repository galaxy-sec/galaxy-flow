use std::time::SystemTime;

use orion_common::friendly::AppendAble;
use serde::ser::Serializer;
use serde::Serialize;
use time::OffsetDateTime;

use crate::ability::prelude::ExecOut;
use crate::execution::action::Action;

#[derive(Debug, Clone, Getters, PartialEq, Serialize)]
pub struct Task {
    name: String,
    #[serde(serialize_with = "serialize_fmt")]
    begin: SystemTime,
    pub stdout: String,
    result: std::result::Result<RunningTime, String>,
    actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunningTime {
    running_time: String,
}

// 序列化进行时间格式化
fn serialize_fmt<S>(value: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let datetime = OffsetDateTime::from(*value);

    datetime
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap()
        .serialize(serializer)
}

impl Task {
    pub fn finish(&mut self) {
        let end = SystemTime::now();
        let duration = end.duration_since(self.begin).unwrap();
        let mut total_nanos = duration.as_nanos();

        let units = [(1_000_000_000, "s"), (1_000_000, "ms")];

        let mut formate_time = String::new();
        for (unit, unit_name) in units {
            let value = total_nanos / unit;
            if value > 0 {
                formate_time.push_str(&format!("{value}{unit_name}"));
            }
            total_nanos %= unit;
        }
        self.result = Ok(RunningTime {
            running_time: formate_time,
        });
    }
    pub fn err(&mut self, msg: String) {
        self.result = Err(msg);
    }
}

impl From<String> for Task {
    fn from(name: String) -> Self {
        Self {
            name,
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
            actions: vec![],
        }
    }
}
impl From<&String> for Task {
    fn from(name: &String) -> Self {
        Self {
            name: name.clone(),
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
            actions: vec![],
        }
    }
}

impl From<&str> for Task {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            begin: SystemTime::now(),
            stdout: String::new(),
            result: Err("unknow".into()),
            actions: vec![],
        }
    }
}

impl AppendAble<Task> for Task {
    fn append(&mut self, task: Task) {
        // 将子任务的执行状态合并到该任务中
        if let Err(task_err) = task.result {
            match &mut self.result {
                Ok(_) => {
                    self.result = Err(task_err);
                }
                Err(e) => {
                    e.push_str(&task_err);
                }
            }
        }
        self.actions.extend(task.actions);
    }
}
impl AppendAble<Action> for Task {
    fn append(&mut self, action: Action) {
        // 将子任务的执行状态合并到该任务中
        if let Err(task_err) = action.result() {
            match &mut self.result {
                Ok(_) => {
                    self.result = Err(task_err.clone());
                }
                Err(e) => {
                    e.push_str(task_err);
                }
            }
        }
        self.actions.push(action);
    }
}
impl AppendAble<ExecOut> for Task {
    fn append(&mut self, ro: ExecOut) {
        match ro {
            ExecOut::Task(t) => self.append(t),
            ExecOut::Action(j) => self.append(j),
            _ => {}
        }
    }
}

#[test]
fn build_action() {
    let mut task1 = Task::from("test.name");
    task1.finish();
    let mut task2 = Task::from("test.name");
    task2.err("bad".into());
}
