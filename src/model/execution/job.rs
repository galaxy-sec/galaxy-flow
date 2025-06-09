use orion_common::friendly::AppendAble;
use serde::Serialize;

use super::{runnable::ExecOut, task::Task};
#[derive(Debug, Clone, Default, Getters, PartialEq, Serialize)]
pub struct Job {
    name: String,
    tasks: Vec<Task>,
}
impl From<&str> for Job {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            tasks: Vec::new(),
        }
    }
}

impl From<&String> for Job {
    fn from(name: &String) -> Self {
        Self {
            name: name.to_string(),
            tasks: Vec::new(),
        }
    }
}

impl AppendAble<Task> for Job {
    fn append(&mut self, task: Task) {
        self.tasks.push(task);
    }
}
impl AppendAble<Job> for Job {
    fn append(&mut self, mut job: Job) {
        self.tasks.append(&mut job.tasks);
    }
}
impl AppendAble<ExecOut> for Job {
    fn append(&mut self, ro: ExecOut) {
        match ro {
            ExecOut::Task(t) => self.append(t),
            ExecOut::Job(j) => self.append(j),
            _ => {}
        }
    }
}
