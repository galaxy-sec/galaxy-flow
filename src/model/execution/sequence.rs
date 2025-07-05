use std::collections::{HashMap, VecDeque};
use std::sync::Arc;

use async_trait::async_trait;
use derive_more::Deref;
use orion_common::friendly::AppendAble;
use orion_error::{ErrorConv, UvsSysFrom};

use crate::ability::prelude::TaskValue;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_flow::meta::{FlowMeta, FlowMetaHold};
use crate::components::gxl_mod::meta::ModMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::hold::{ComHold, IsolationHold};
use crate::execution::job::Job;
use crate::execution::runnable::ComponentMeta;
use crate::execution::runnable::{AsyncRunnableTrait, ExecOut, TaskResult};
use crate::execution::task::Task;
use crate::execution::VarSpace;
use crate::meta::{GxlMeta, MetaInfo};
use crate::ExecError;

use super::hold::TransableHold;
#[derive(Debug, Clone, Getters)]
pub struct LableGuard {
    lable: RunLable,
    open: bool,
}
impl Drop for LableGuard {
    fn drop(&mut self) {
        self.open = false;
    }
}
impl LableGuard {
    pub fn from_entry(value: &FlowMeta) -> Self {
        Self {
            lable: RunLable::Entry(value.long_name()),
            open: true,
        }
    }

    pub fn from_exit(value: &FlowMeta) -> Self {
        Self {
            lable: RunLable::Exist(value.long_name()),
            open: true,
        }
    }
    pub fn from_mod(value: &ModMeta) -> Self {
        Self {
            lable: RunLable::ModProp(value.long_name()),
            open: true,
        }
    }
    pub fn from_flow() -> Self {
        Self {
            lable: RunLable::Flow,
            open: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunLable {
    Entry(String),
    Exist(String),
    ModProp(String),
    Flow,
}
#[derive(Clone, Getters, Default)]
pub struct Sequence {
    name: String,
    only_items: HashMap<RunLable, bool>,
    run_items: Vec<ComHold>,
}

#[derive(Clone, Getters, Default, Deref)]
pub struct ExecUnit {
    run_items: Vec<ComHold>,
}

impl From<&str> for Sequence {
    fn from(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

impl Sequence {
    pub async fn execute(&self, ctx: ExecContext, def: VarSpace, spc: &GxlSpace) -> TaskResult {
        self.execute_sequence(ctx, def, spc).await
    }

    pub async fn test_execute(
        &self,
        ctx: ExecContext,
        def: VarSpace,
        spc: &GxlSpace,
    ) -> TaskResult {
        self.execute_sequence(ctx, def, spc).await
    }

    async fn execute_sequence(
        &self,
        ctx: ExecContext,
        mut def: VarSpace,
        spc: &GxlSpace,
    ) -> TaskResult {
        let mut job = Job::from(&self.name);
        let mut undo_stack = VecDeque::new();
        warn!(target: ctx.path(), "sequence size: {}  dryrun: {}", self.run_items().len(), ctx.dryrun());

        let mut transaction_begin = false;
        for (index, item) in self.run_items.iter().enumerate() {
            info!(target: ctx.path(), "executing item {}: {} ", index, item.com_meta().full_name());
            if item.is_transaction() {
                transaction_begin = true;
                warn!(target: ctx.path(), "transaction begin")
            }
            if transaction_begin {
                if let Some(undo) = item.undo_hold() {
                    let mut sequ = Sequence::default();
                    spc.load_flow_by(&undo, &mut sequ).err_conv()?;
                    for undo in sequ.run_items() {
                        info!(target: ctx.path(), "regist undo {}", undo.com_meta().name());
                        undo_stack.push_back((undo.clone(), def.clone()));
                    }
                }
            }
            let mut sub_queue = VecDeque::new();

            if *ctx.dryrun() {
                if let Some(dryrun_meta) = item.dryrun_hold() {
                    let mut sequ = Sequence::default();
                    spc.load_flow_by(&dryrun_meta, &mut sequ).err_conv()?;
                    for dryrun in sequ.run_items() {
                        info!(target: ctx.path(), "regist undo {}", dryrun.com_meta().name());
                        sub_queue.push_back(dryrun.clone());
                    }
                } else {
                    sub_queue.push_back(item.clone());
                }
            } else {
                sub_queue.push_back(item.clone());
            };
            while let Some(item) = sub_queue.pop_back() {
                match item.async_exec(ctx.clone(), def.clone()).await {
                    Ok(TaskValue { vars, rec, .. }) => {
                        def = vars;
                        job.append(rec);
                    }
                    Err(e) => {
                        warn!("Sequence aborted at step {index}: {e}");
                        warn!("will execute undo :{}", undo_stack.len());
                        self.undo_transactions(ctx.clone(), undo_stack).await;
                        return Err(e);
                    }
                }
            }
        }

        Ok(TaskValue::from((def, ExecOut::Job(job))))
    }

    async fn undo_transactions(
        &self,
        ctx: ExecContext,
        mut undo_stack: VecDeque<(ComHold, VarSpace)>,
    ) {
        while let Some((undo, dict)) = undo_stack.pop_back() {
            match undo.async_exec(ctx.clone(), dict).await {
                Ok(_) => warn!("Undo successful for {}", undo.com_meta().name()),
                Err(e) => error!("Undo failed for {}: {}", undo.com_meta().name(), e),
            }
        }
    }
    pub fn append_trans_hold<H: Into<TransableHold>>(&mut self, guard: &LableGuard, hold: H) {
        let trans_hold = hold.into();
        debug_assert!(trans_hold.assembled());
        debug!(target:"exec/sque", "only_item :{:?}", guard.lable());
        if !self.only_items().contains_key(guard.lable()) || *guard.open() {
            info!(target:"exec/sque", "only_item :{}", trans_hold.com_meta().full_name());
            self.only_items.insert(guard.lable().clone(), true);
            self.run_items.push(AsyncComHold::from(trans_hold).into());
        }
    }
}

impl AppendAble<AsyncComHold> for Sequence {
    fn append(&mut self, node: AsyncComHold) {
        debug!(target: "exec/sque", "append {}", node.com_meta().full_name() );
        self.run_items.push(node.into());
    }
}

impl AppendAble<IsolationHold> for Sequence {
    fn append(&mut self, node: IsolationHold) {
        debug!(target: "exec/sque", "append {}", node.hold().com_meta().full_name() );
        self.run_items.push(node.into());
    }
}

#[derive(Clone, Default)]
pub struct RunStub {
    name: String,
    trans_begin: bool,
    undo_item: Vec<TransableHold>,
    should_fail: bool,
    effect: Option<Arc<dyn Fn() + Send + Sync>>,
}

impl RunStub {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }
    pub fn with_transaction(mut self, trans_begin: bool, undo: TransableHold) -> Self {
        self.trans_begin = trans_begin;
        self.undo_item = vec![undo];
        self
    }
    pub fn with_transactions(mut self, trans_begin: bool, undo: Vec<TransableHold>) -> Self {
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

    fn undo_hold(&self) -> Option<FlowMetaHold> {
        //self.undo_item.clone()
        todo!();
    }
}

impl ComponentMeta for RunStub {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}

#[async_trait]
impl AsyncRunnableTrait for RunStub {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
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

        let spc = GxlSpace::default();
        let TaskValue { rec, .. } = flow.execute(ctx, def, &spc).await.unwrap();

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
        let step1 = stub_node("step1").with_transaction(true, TransableHold::from(step1_undo));

        let step2 = stub_node("step2")
            .with_transaction(false, TransableHold::from(step2_undo))
            .with_should_fail(true); // 使第二步失败

        flow.append(AsyncComHold::from(step1));
        flow.append(AsyncComHold::from(step2));

        // 执行并验证
        let spc = GxlSpace::default();
        let result = flow.execute(ctx, def, &spc).await;
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
