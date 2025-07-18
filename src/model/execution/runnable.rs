use async_trait::async_trait;
use serde::Serialize;

use crate::context::ExecContext;
use crate::execution::task::Task;
use crate::meta::GxlMeta;
use crate::ExecResult;

use super::action::Action;
use super::job::Job;
use super::VarSpace;
pub type PipeSender = std::sync::mpsc::Sender<String>;
pub type PipeReceiver = std::sync::mpsc::Receiver<String>;
pub type Pipe = (PipeReceiver, PipeSender);

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum ExecOut {
    Action(Action),
    Task(Task),
    Job(Job),
    Ignore,
    Code(i32),
}
#[derive(Debug, Clone, PartialEq)]
pub struct TaskValue {
    pub vars: VarSpace,
    pub out: String,
    pub rec: ExecOut,
}
impl TaskValue {
    pub fn new(vars: VarSpace, out: String, task: ExecOut) -> Self {
        Self {
            vars,
            out,
            rec: task,
        }
    }
    pub fn rec(&self) -> &ExecOut {
        &self.rec
    }
    pub fn append_out(&mut self, out: String) {
        self.out.push_str(&out);
    }
}
impl From<(VarSpace, ExecOut)> for TaskValue {
    fn from(value: (VarSpace, ExecOut)) -> Self {
        Self {
            vars: value.0,
            rec: value.1,
            out: String::new(),
        }
    }
}

impl From<(VarSpace, ExecOut, String)> for TaskValue {
    fn from(value: (VarSpace, ExecOut, String)) -> Self {
        Self {
            vars: value.0,
            rec: value.1,
            out: value.2,
        }
    }
}

pub type VarsResult = ExecResult<VarSpace>;
pub type TaskResult = ExecResult<TaskValue>;

//#[automock]
#[async_trait]
pub trait AsyncRunnableTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> TaskResult;
}

pub trait RunnableTrait {
    fn exec(&self, ctx: ExecContext, dict: VarSpace) -> TaskResult;
}

pub trait ComponentMeta {
    fn gxl_meta(&self) -> GxlMeta;
}
pub trait MetaInfo: AsyncRunnableTrait {
    fn meta(&self) -> GxlMeta;
}
