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

pub type TaskResult = ExecResult<ExecOut>;
pub type VarsResult = ExecResult<VarSpace>;
pub type VTResult = ExecResult<(VarSpace, ExecOut)>;

//#[automock]
#[async_trait]
pub trait AsyncRunnableTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult;
}

#[async_trait]
pub trait AsyncDryrunRunnableTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace, dryrun: bool) -> VTResult;
}

pub trait RunnableTrait {
    fn exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult;
}

pub trait ComponentMeta {
    fn com_meta(&self) -> GxlMeta;
}
pub trait MetaInfo: AsyncRunnableTrait {
    fn meta(&self) -> GxlMeta;
}
