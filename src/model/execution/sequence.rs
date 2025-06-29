use std::collections::VecDeque;
use std::sync::Arc;

use async_trait::async_trait;
use orion_common::friendly::AppendAble;
use orion_error::UvsSysFrom;

use crate::ability::prelude::TaskValue;
use crate::annotation::{Dryrunable, Transaction};
use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::hold::{ComHold, IsolationHold};
use crate::execution::job::Job;
use crate::execution::runnable::ComponentMeta;
use crate::execution::runnable::{AsyncRunnableTrait, ExecOut, VTResult};
use crate::execution::task::Task;
use crate::execution::VarSpace;
use crate::meta::GxlMeta;
use crate::ExecError;

use super::hold::TransableHold;

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
        warn!(target: ctx.path(), "sequence size: {}  dryrun: {}", self.run_items().len(), ctx.dryrun());

        let mut transaction_begin = false;
        for (index, item) in self.run_items.iter().enumerate() {
            debug!(target: ctx.path(), "executing item {}: {} ", index, item.com_meta().name());
            if item.is_transaction() {
                transaction_begin = true;
                warn!(target: ctx.path(), "transaction begin")
            }
            if transaction_begin {
                if let Some(undo) = item.undo_hold() {
                    info!(target: ctx.path(), "regist undo {}", undo.com_meta().name());
                    undo_stack.push_back((undo, def.clone()));
                }
            }
            let result = if *ctx.dryrun() {
                if let Some(dryrun) = item.dryrun_hold() {
                    warn!(target: ctx.path(), "execute dryrun flow");
                    dryrun.async_exec(ctx.clone(), def.clone()).await
                } else {
                    item.async_exec(ctx.clone(), def.clone()).await
                }
            } else {
                item.async_exec(ctx.clone(), def.clone()).await
            };
            match result {
                Ok(TaskValue { vars, rec, .. }) => {
                    def = vars;
                    job.append(rec);
                }
                Err(e) => {
                    warn!("Sequence aborted at step {}: {}", index, e);
                    warn!("will execute undo :{}", undo_stack.len());
                    self.undo_transactions(ctx.clone(), undo_stack).await;
                    return Err(e);
                }
            }
        }

        Ok(TaskValue::from((def, ExecOut::Job(job))))
    }

    async fn undo_transactions(
        &self,
        ctx: ExecContext,
        mut undo_stack: VecDeque<(TransableHold, VarSpace)>,
    ) {
        while let Some((undo, dict)) = undo_stack.pop_back() {
            match undo.async_exec(ctx.clone(), dict).await {
                Ok(_) => warn!("Undo successful for {}", undo.com_meta().name()),
                Err(e) => error!("Undo failed for {}: {}", undo.com_meta().name(), e),
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
    trans_begin: bool,
    undo_item: Option<TransableHold>,
    should_fail: bool,
    effect: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl RunStub {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            trans_begin: false,
            undo_item: None,
            should_fail: false,
            effect: None,
        }
    }
    pub fn with_transaction(mut self, trans_begin: bool, undo: Option<TransableHold>) -> Self {
        self.trans_begin = trans_begin;
        self.undo_item = undo;
        self
    }

    pub fn with_should_fail(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    // 直接添加副作用到 RunStub
    pub fn with_effect<F>(mut self, effect: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.effect = Some(Arc::new(effect));
        self
    }
}

impl Transaction for RunStub {
    fn is_transaction(&self) -> bool {
        self.trans_begin
    }

    fn undo_hold(&self) -> Option<TransableHold> {
        self.undo_item.clone()
    }
}

impl ComponentMeta for RunStub {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}

#[async_trait]
impl AsyncRunnableTrait for RunStub {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        // 先执行可能的副作用
        if let Some(effect) = &self.effect {
            effect();
        }

        // 然后处理正常执行逻辑
        if self.should_fail {
            //return Err(anyhow::anyhow!("Intentional failure in {}", self.name));
            return Err(ExecError::from_sys("intentional failure".into()));
        }

        debug!(target: ctx.path(), "executing stub: {}", self.name);
        let task = Task::from(&self.name);
        Ok(TaskValue::from((def, ExecOut::Task(task))))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use super::*;
    use crate::execution::exec_init_env;

    fn stub_node(name: &str) -> RunStub {
        RunStub::new(name)
    }

    #[tokio::test]
    async fn sequence_execution() {
        let (ctx, def) = exec_init_env();
        let mut flow = Sequence::from("test.flow");

        flow.append(AsyncComHold::from(stub_node("step1")));
        flow.append(AsyncComHold::from(stub_node("step2")));

        let TaskValue { rec, .. } = flow.execute(ctx, def).await.unwrap();

        if let ExecOut::Job(job) = rec {
            assert_eq!(job.tasks().len(), 2);
            assert_eq!(job.tasks()[0].name(), "step1");
            assert_eq!(job.tasks()[1].name(), "step2");
        } else {
            panic!("Expected Job output");
        }
    }

    #[tokio::test]
    async fn verify_undo_execution() {
        let (ctx, def) = exec_init_env();
        let mut flow = Sequence::from("verify_undo.flow");

        // 创建执行追踪器
        let undo1_executed = Arc::new(Mutex::new(false));
        let undo2_executed = Arc::new(Mutex::new(false));

        // 直接为 RunStub 添加 effect
        let step1_undo = stub_node("undo_step1").with_effect({
            let flag = undo1_executed.clone();
            move || *flag.lock().unwrap() = true
        });

        let step2_undo = stub_node("undo_step2").with_effect({
            let flag = undo2_executed.clone();
            move || *flag.lock().unwrap() = true
        });

        // 配置事务
        let step1 = stub_node("step1")
            .with_transaction(true, Some(TransableHold::from(Arc::new(step1_undo))));

        let step2 = stub_node("step2")
            .with_transaction(false, Some(TransableHold::from(Arc::new(step2_undo))))
            .with_should_fail(true); // 使第二步失败

        flow.append(AsyncComHold::from(step1));
        flow.append(AsyncComHold::from(step2));

        // 执行并验证
        let result = flow.execute(ctx, def).await;
        assert!(result.is_err(), "Execution should fail");

        // 检查 undo 是否执行
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
