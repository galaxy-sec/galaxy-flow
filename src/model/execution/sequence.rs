use async_trait::async_trait;
use orion_common::friendly::AppendAble;
use orion_common::friendly::MultiNew2;

use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::job::Job;
use crate::execution::runnable::AsyncRunnableTrait;
use crate::execution::runnable::ExecOut;
use crate::execution::runnable::RunnableTrait;
use crate::execution::task::Task;
use crate::meta::GxlMeta;
use crate::meta::GxlType;

use super::hold::ComHold;
use super::hold::IsolationHold;
use super::runnable::ComponentMeta;
use super::runnable::VTResult;
use super::runnable::VarSpace;

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
        let mut job = Job::from(&self.name);
        warn!(target: ctx.path() ,"sequ size:{} ", self.run_items().len());
        for obj in &self.run_items {
            debug!(target: ctx.path() ,"sequ exec runner :{} ",obj.com_meta().name());
            let (cur_dict, out) = obj.async_exec(ctx.clone(), def).await?;
            def = cur_dict;
            job.append(out);
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
}
impl From<&str> for RunStub {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
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
        GxlMeta::new2(GxlType::Ignore, "stub")
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
