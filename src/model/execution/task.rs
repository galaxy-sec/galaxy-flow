use crate::util::serialize_time_format::serialize_time_format;
use orion_common::friendly::AppendAble;
use serde::Serialize;
use time::OffsetDateTime;

use crate::ability::prelude::ExecOut;
use crate::execution::action::Action;

#[derive(Debug, Clone, Getters, PartialEq, Serialize)]
pub struct Task {
    name: String,
    #[serde(serialize_with = "serialize_time_format")]
    begin: OffsetDateTime,
    pub stdout: String,
    pub result: Result<RunningTime, String>,
    actions: Vec<Action>,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct RunningTime {
    running_time: String,
}

impl Task {
    pub fn finish(&mut self) {
        let end = OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc());
        let mut total_nanos = (end - self.begin).whole_nanoseconds();

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
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
            stdout: String::new(),
            result: Ok(RunningTime {
                running_time: String::new(),
            }),
            actions: vec![],
        }
    }
}
impl From<&String> for Task {
    fn from(name: &String) -> Self {
        Self {
            name: name.clone(),
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
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
            begin: OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc()),
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
        self.stdout.push_str(&task.stdout);
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
