use std::collections::HashMap;

use async_trait::async_trait;
use derive_more::From;

use crate::context::ExecContext;
use crate::meta::RgoMeta;
use crate::var::VarsDict;
use crate::ExecResult;

use super::job::Job;
use super::task::Task;
pub type PipeSender = std::sync::mpsc::Sender<String>;
pub type PipeReceiver = std::sync::mpsc::Receiver<String>;
pub type Pipe = (PipeReceiver, PipeSender);

#[derive(Debug, Clone, PartialEq)]
pub enum ExecOut {
    Task(Task),
    Job(Job),
    Ignore,
    Code(i32),
}

/*
impl Display for ExecOut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}
*/

pub type TaskResult = ExecResult<ExecOut>;
pub type VarsResult = ExecResult<VarsDict>;
pub type VTResult = ExecResult<(VarsDict, ExecOut)>;

#[derive(Debug, Clone, Default, PartialEq, From, Getters)]
pub struct VarSpace {
    globle: VarsDict,
    nameds: HashMap<String, VarsDict>,
}
#[derive(Debug, Clone, Default, PartialEq, From)]
pub enum DictUse {
    #[default]
    Global,
    Named(String),
}
//#[automock]
#[async_trait]
pub trait AsyncRunnableTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: VarsDict) -> VTResult;
}
pub trait RunnableTrait {
    fn exec(&self, ctx: ExecContext, dict: VarsDict) -> VTResult;
}

pub trait ComponentMeta {
    fn com_meta(&self) -> RgoMeta;
}
pub trait MetaInfo: AsyncRunnableTrait {
    fn meta(&self) -> RgoMeta;
}
/*
pub fn channel_pass_data(recv: &PipeReceiver, send: &PipeSender) -> VTResult {
    while let Ok(data) = recv.try_recv() {
        send.send(data).owe_sys()?;
    }
    Ok(ExecOut::Ignore)
}
*/

//pub type RunHold = std::sync::Arc<dyn AsyncRunnableTrait>;
/*
#[async_trait]
pub trait AsyncRunable: Sync + Send {
    async fn async_forword(&self, ctx: ExecContext, dct: VarsDict) -> EResult;
}

pub fn make_run_hold<T: AsyncRunnableTrait + 'static>(obj: T) -> RunHold {
    Arc::new(obj)
}
*/
