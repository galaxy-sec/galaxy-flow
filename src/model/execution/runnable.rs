use std::sync::Arc;

use async_trait::async_trait;
use orion_error::ErrorOwe;

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

pub type EOResult = ExecResult<ExecOut>;
//#[automock]
#[async_trait]
pub trait AsyncRunnableTrait {
    async fn async_exec(&self, ctx: ExecContext, dict: &VarsDict) -> EOResult;
}
//#[automock]
pub trait ComponentRunnable: AsyncRunnableTrait {
    fn meta(&self) -> RgoMeta;
}
pub trait MetaInfo: AsyncRunnableTrait {
    fn meta(&self) -> RgoMeta;
}
pub fn channel_pass_data(recv: &PipeReceiver, send: &PipeSender) -> EOResult {
    while let Ok(data) = recv.try_recv() {
        send.send(data).owe_sys()?;
    }
    Ok(ExecOut::Ignore)
}

pub type RunHold = std::sync::Arc<dyn AsyncRunnableTrait>;
pub type ComHold = std::sync::Arc<dyn ComponentRunnable>;

/*
#[async_trait]
pub trait AsyncRunable: Sync + Send {
    async fn async_forword(&self, ctx: ExecContext, dct: VarsDict) -> EResult;
}
*/
//pub type ARunHold = std::sync::Arc<dyn AsyncRunable>;

pub fn make_run_hold<T: AsyncRunnableTrait + 'static>(obj: T) -> RunHold {
    Arc::new(obj)
}

pub fn make_stc_hold<T: ComponentRunnable + 'static>(obj: T) -> ComHold {
    Arc::new(obj)
}
