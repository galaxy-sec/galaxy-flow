use std::collections::VecDeque;

use async_trait::async_trait;
use orion_common::friendly::AppendAble;

use crate::annotation::FlowHold;
use crate::annotation::Transaction;
use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::job::Job;
use crate::execution::runnable::AsyncRunnableTrait;
use crate::execution::runnable::ExecOut;
use crate::execution::runnable::RunnableTrait;
use crate::execution::task::Task;
use crate::meta::GxlMeta;

use super::hold::ComHold;
use super::hold::IsolationHold;
use super::runnable::ComponentMeta;
use super::runnable::VTResult;
use super::VarSpace;

#[derive(Clone, Getters)]
pub struct Sequence {
    name: String,
    run_items: Vec<ComHold>,
}

impl From<&str> for Sequence {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            run_items: Vec::new(),
        }
    }
}

impl Sequence {
    pub async fn execute(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.forword(ctx, def).await
    }
    pub async fn test_execute(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.forword(ctx, def).await
    }

    async fn forword(&self, ctx: ExecContext, mut def: VarSpace) -> VTResult {
        let mut undo_items: VecDeque<(FlowHold, VarSpace)> = VecDeque::new();
        let mut job = Job::from(&self.name);
        warn!(target: ctx.path() ,"sequ size:{} ", self.run_items().len());
        let mut do_error = None;
        for obj in &self.run_items {
            debug!(target: ctx.path() ,"sequ exec runner :{} ",obj.com_meta().name());
            if obj.is_transaction() {
                if let Some(undo) = obj.undo_flow() {
                    undo_items.push_back((undo, def.clone()));
                }
            }

            match obj.async_exec(ctx.clone(), def.clone()).await {
                Ok((cur_dict, out)) => {
                    def = cur_dict;
                    job.append(out);
                }
                Err(e) => {
                    do_error = Some(e);
                    break;
                }
            }
        }
        if let Some(e) = do_error {
            while let Some((undo, dict)) = undo_items.pop_back() {
                match undo.async_exec(ctx.clone(), dict).await {
                    Ok(_) => {
                        warn!("undo :{} success!", undo.m_name());
                    }
                    Err(e) => {
                        error!("undo :{} fail \n{}!", undo.m_name(), e);
                    }
                }
            }
            //TODO: report;
            return Err(e);
        }
        Ok((def, ExecOut::Job(job)))
    }
}

impl AppendAble<AsyncComHold> for Sequence {
    fn append(&mut self, node: AsyncComHold) {
        self.run_items.push(node.into());
    }
}
impl AppendAble<IsolationHold> for Sequence {
    fn append(&mut self, node: IsolationHold) {
        self.run_items.push(node.into());
    }
}

#[derive(Clone)]
pub struct RunStub {
    name: String,
    is_trans: bool,
    undo_item: Option<FlowHold>,
}
impl RunStub {
    pub fn with_transaction(mut self, is_trans: bool, undo: Option<FlowHold>) -> Self {
        self.is_trans = is_trans;
        self.undo_item = undo;
        self
    }
}
impl Transaction for RunStub {
    fn is_transaction(&self) -> bool {
        self.is_trans
    }

    fn undo_flow(&self) -> Option<FlowHold> {
        self.undo_item.clone()
    }
}
impl From<&str> for RunStub {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            is_trans: false,
            undo_item: None,
        }
    }
}
#[async_trait]
impl AsyncRunnableTrait for RunStub {
    async fn async_exec(&self, ctx: ExecContext, _def: VarSpace) -> VTResult {
        debug!(target:ctx.path(), "{}", self.name);
        let task = Task::from(&self.name);
        Ok((_def, ExecOut::Task(task)))
    }
}

impl RunnableTrait for RunStub {
    fn exec(&self, ctx: ExecContext, _def: VarSpace) -> VTResult {
        debug!(target:ctx.path(), "{}", self.name);
        let task = Task::from(&self.name);
        Ok((_def, ExecOut::Task(task)))
    }
}
impl ComponentMeta for RunStub {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::exec_init_env;

    use super::*;
    fn stub_node(name: &str) -> RunStub {
        RunStub::from(name)
    }
    #[tokio::test]
    async fn build_flow() {
        let (ctx, def) = exec_init_env();

        let mut flow = Sequence::from("test.flow");
        let node21 = stub_node("self.step1");
        let node22 = stub_node("self.step2");
        flow.append(AsyncComHold::from(node21.clone()));
        flow.append(AsyncComHold::from(node22.clone()));

        let (_, out) = flow.execute(ctx, def).await.unwrap();
        if let ExecOut::Job(job) = out {
            debug!("{:#?}", job);
            assert_eq!(job.tasks().len(), 2);
            assert_eq!(job.tasks()[0].name(), "self.step1");
            assert_eq!(job.tasks()[1].name(), "self.step2");
        }
    }
}
