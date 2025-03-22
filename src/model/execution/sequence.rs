use orion_common::friendly::AppendAble;
use orion_common::friendly::MultiNew2;

use crate::context::ExecContext;
use crate::execution::job::Job;
use crate::execution::runnable::ComHold;
use crate::execution::runnable::ComponentRunnable;
use crate::execution::runnable::EOResult;
use crate::execution::runnable::ExecOut;
use crate::execution::runnable::RunnableTrait;
use crate::execution::task::Task;
use crate::meta::GxlType;
use crate::meta::RgoMeta;
use crate::var::VarsDict;

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
    pub fn execute(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        self.forword(ctx, def)
    }
    pub fn test_execute(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        self.forword(ctx, def)
    }

    fn forword(&self, ctx: ExecContext, def: &mut VarsDict) -> EOResult {
        let mut job = Job::from(&self.name);
        for obj in &self.run_items {
            debug!(target: ctx.path() ,"sequ exec runner :{} ",obj.meta().name());
            let out = obj.exec(ctx.clone(), def)?;
            job.append(out);
        }
        Ok(ExecOut::Job(job))
    }
}

impl AppendAble<ComHold> for Sequence {
    fn append(&mut self, node: ComHold) {
        self.run_items.push(node);
    }
}

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
impl RunnableTrait for RunStub {
    fn exec(&self, ctx: ExecContext, _def: &mut VarsDict) -> EOResult {
        debug!(target:ctx.path(), "{}", self.name);
        let task = Task::from(&self.name);
        Ok(ExecOut::Task(task))
    }
}
impl ComponentRunnable for RunStub {
    fn meta(&self) -> RgoMeta {
        RgoMeta::new2(GxlType::Ignore, "stub")
    }
}

#[cfg(test)]
mod tests {
    use crate::execution::{exec_init_env, runnable::ComponentRunnable};

    use super::*;
    use std::sync::Arc;
    fn stub_node(name: &str) -> Arc<dyn ComponentRunnable> {
        Arc::new(RunStub::from(name))
    }
    #[test]
    fn build_flow() {
        let (ctx, mut def) = exec_init_env();

        let mut flow = Sequence::from("test.flow");
        let node21 = stub_node("self.step1");
        let node22 = stub_node("self.step2");
        flow.append(node21.clone());
        flow.append(node22.clone());

        let out = flow.execute(ctx, &mut def);
        if let ExecOut::Job(job) = out.unwrap() {
            debug!("{:#?}", job);
            assert_eq!(job.tasks().len(), 2);
            assert_eq!(job.tasks()[0].name(), "self.step1");
            assert_eq!(job.tasks()[1].name(), "self.step2");
        }
    }
}
