use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;
use orion_common::friendly::AppendAble;

use crate::annotation::FlowHold;
use crate::annotation::Transaction;
use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::hold::{ComHold, IsolationHold};
use crate::execution::job::Job;
use crate::execution::runnable::ComponentMeta;
use crate::execution::runnable::{AsyncRunnableTrait, ExecOut, RunnableTrait, VTResult};
use crate::execution::task::Task;
use crate::execution::VarSpace;
use crate::meta::GxlMeta;

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
        self.execute_sequence(ctx, def).await
    }

    pub async fn test_execute(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.execute_sequence(ctx, def).await
    }

    async fn execute_sequence(&self, ctx: ExecContext, mut def: VarSpace) -> VTResult {
        let mut job = Job::from(&self.name);
        let mut undo_stack = VecDeque::new();
        warn!(target: ctx.path(), "sequence size: {}", self.run_items().len());

        for (index, item) in self.run_items.iter().enumerate() {
            debug!(target: ctx.path(), "executing item {}: {}", index, item.com_meta().name());

            match item.async_exec(ctx.clone(), def.clone()).await {
                Ok((new_def, out)) => {
                    def = new_def;
                    job.append(out);

                    // Record successful transaction for potential undo
                    if item.is_transaction() {
                        if let Some(undo) = item.undo_flow() {
                            undo_stack.push_back((undo, def.clone()));
                        }
                    }
                }
                Err(e) => {
                    warn!("Sequence aborted at step {}: {}", index, e);
                    self.undo_transactions(ctx.clone(), undo_stack).await;
                    return Err(e);
                }
            }
        }

        Ok((def, ExecOut::Job(job)))
    }

    async fn undo_transactions(
        &self,
        ctx: ExecContext,
        mut undo_stack: VecDeque<(FlowHold, VarSpace)>,
    ) {
        while let Some((undo, dict)) = undo_stack.pop_back() {
            match undo.async_exec(ctx.clone(), dict).await {
                Ok(_) => warn!("Undo successful for {}", undo.m_name()),
                Err(e) => error!("Undo failed for {}: {}", undo.m_name(), e),
            }
        }
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
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        debug!(target: ctx.path(), "executing stub: {}", self.name);
        let task = Task::from(&self.name);
        Ok((def, ExecOut::Task(task)))
    }
}

impl RunnableTrait for RunStub {
    fn exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        debug!(target: ctx.path(), "executing stub: {}", self.name);
        let task = Task::from(&self.name);
        Ok((def, ExecOut::Task(task)))
    }
}

impl ComponentMeta for RunStub {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}
/*
impl RunStub {
    fn with_custom_effect<F>(mut self, effect: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        // Store effect in a type-erased form
        self.undo_item = Some(FlowHold::from(AsyncComHold::from(EffectStub {
            name: self.name.clone(),
            effect: Arc::new(effect),
        })));
        self
    }
}

// Helper struct for custom effects
struct EffectStub {
    name: String,
    effect: Arc<dyn Fn() + Send + Sync>,
}

#[async_trait]
impl AsyncRunnableTrait for EffectStub {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        debug!(target: ctx.path(), "executing effect: {}", self.name);
        (self.effect)();
        Ok((def, ExecOut::Task(Task::from(&self.name))))
    }
}

impl ComponentMeta for EffectStub {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("effect_stub")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution::exec_init_env;
    use std::sync::{Arc, Mutex};

    fn stub_node(name: &str) -> RunStub {
        RunStub::from(name)
    }

    #[tokio::test]
    async fn sequence_execution() {
        let (ctx, def) = exec_init_env();
        let mut flow = Sequence::from("test.flow");

        flow.append(AsyncComHold::from(stub_node("step1")));
        flow.append(AsyncComHold::from(stub_node("step2")));

        let (_, out) = flow.execute(ctx, def).await.unwrap();

        if let ExecOut::Job(job) = out {
            assert_eq!(job.tasks().len(), 2);
            assert_eq!(job.tasks()[0].name(), "step1");
            assert_eq!(job.tasks()[1].name(), "step2");
        } else {
            panic!("Expected Job output");
        }
    }

    #[tokio::test]
    async fn transaction_rollback_on_failure() {
        let (ctx, def) = exec_init_env();
        let mut flow = Sequence::from("transaction.flow");

        // Step 1: Successful transaction
        let step1 =
            stub_node("step1").with_transaction(true, Some(FlowHold::new(stub_node("undo_step1"))));

        // Step 2: Transaction that will fail
        let step2 = stub_node("step2")
            .with_transaction(true, Some(FlowHold::new(stub_node("undo_step2"))))
            .with_should_fail(true);

        // Step 3: Should never execute
        let step3 = stub_node("step3");

        flow.append(AsyncComHold::from(step1));
        flow.append(AsyncComHold::from(step2));
        flow.append(AsyncComHold::from(step3));

        let result = flow.execute(ctx, def).await;
        assert!(result.is_err(), "Execution should fail");
    }

    #[tokio::test]
    async fn verify_undo_execution() {
        let (ctx, def) = exec_init_env();
        let mut flow = Sequence::from("verify_undo.flow");

        // Create undo stubs with execution trackers
        let undo1_executed = Arc::new(Mutex::new(false));
        let undo2_executed = Arc::new(Mutex::new(false));

        let undo1 = {
            let flag = undo1_executed.clone();
            stub_node("undo_step1").with_custom_effect(move || *flag.lock().unwrap() = true)
        };

        let undo2 = {
            let flag = undo2_executed.clone();
            stub_node("undo_step2").with_custom_effect(move || *flag.lock().unwrap() = true)
        };

        // Successful transaction
        let step1 = stub_node("step1").with_transaction(true, Some(FlowHold::from(undo1)));

        // Failing transaction
        let step2 = stub_node("step2")
            .with_transaction(true, Some(FlowHold::from(undo2)))
            .with_should_fail(true);

        flow.append(AsyncComHold::from(step1));
        flow.append(AsyncComHold::from(step2));

        let result = flow.execute(ctx, def).await;
        assert!(result.is_err(), "Execution should fail");

        // Verify undo operations executed in reverse order
        assert!(
            *undo2_executed.lock().unwrap(),
            "undo_step2 should be executed"
        );
        assert!(
            *undo1_executed.lock().unwrap(),
            "undo_step1 should be executed"
        );
    }
}
*/
