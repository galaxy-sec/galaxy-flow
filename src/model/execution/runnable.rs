use std::sync::mpsc::Sender;

use async_trait::async_trait;
use serde::Serialize;

use crate::ability::delegate::GxlAParams;
use crate::context::ExecContext;
use crate::execution::task::Task;
use crate::meta::GxlMeta;
use crate::primitive::GxlAParam;
use crate::types::Property;
use crate::util::redirect::ReadSignal;
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

    pub rec: ExecOut,
}
impl TaskValue {
    pub fn new(vars: VarSpace, task: ExecOut) -> Self {
        Self { vars, rec: task }
    }
    pub fn rec(&self) -> &ExecOut {
        &self.rec
    }
}
impl From<(VarSpace, ExecOut)> for TaskValue {
    fn from(value: (VarSpace, ExecOut)) -> Self {
        Self {
            vars: value.0,
            rec: value.1,
        }
    }
}

impl From<(VarSpace, ExecOut, String)> for TaskValue {
    fn from(value: (VarSpace, ExecOut, String)) -> Self {
        Self {
            vars: value.0,
            rec: value.1,
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

#[async_trait]
pub trait AsyncRunnableArgsTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace, args: &GxlAParams) -> TaskResult;
}

#[async_trait]
pub trait AsyncRunnableWithSenderTrait {
    async fn async_exec(
        &self,
        ctx: ExecContext,
        dict: VarSpace,
        sender: Option<Sender<ReadSignal>>,
    ) -> TaskResult;
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
