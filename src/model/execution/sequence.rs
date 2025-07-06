use std::collections::{HashMap, VecDeque};

use async_trait::async_trait;
use derive_more::Deref;
use orion_common::friendly::AppendAble;
use orion_error::ErrorConv;

use crate::ability::prelude::TaskValue;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_flow::meta::FlowMeta;
use crate::components::gxl_mod::meta::ModMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::context::ExecContext;
use crate::execution::hold::AsyncComHold;
use crate::execution::hold::{ComHold, IsolationHold};
use crate::execution::job::Job;
use crate::execution::runnable::ComponentMeta;
use crate::execution::runnable::{AsyncRunnableTrait, ExecOut, TaskResult};
use crate::execution::task::Task;
use crate::execution::trans::{ComTrans, TransactionManager};
use crate::execution::VarSpace;
use crate::meta::{GxlMeta, MetaInfo};

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
        warn!(target: ctx.path(), "sequence size: {}  dryrun: {}", self.run_items().len(), ctx.dryrun());

        let mut trans_manage = ComTrans::new();
        for (index, item) in self.run_items.iter().enumerate() {
            info!(target: ctx.path(), "executing item {}: {} ", index, item.gxl_meta().full_name());
            if item.is_transaction() {
                trans_manage.begin_transaction();
                warn!(target: ctx.path(), "transaction begin")
            }
            if trans_manage.in_transaction() {
                if let Some(undo) = item.undo_hold() {
                    let mut sequ = Sequence::default();
                    spc.load_flow_by(&undo, &mut sequ).err_conv()?;
                    for undo in sequ.run_items() {
                        info!(target: ctx.path(), "regist undo {}", undo.gxl_meta().name());
                        trans_manage.add_undo_task(undo.clone(), def.clone());
                        //undo_stack.push_back((undo.clone(), def.clone()));
                    }
                }
            }
            let mut sub_queue = VecDeque::new();

            if *ctx.dryrun() {
                if let Some(dryrun_meta) = item.dryrun_hold() {
                    let mut sequ = Sequence::default();
                    spc.load_flow_by(&dryrun_meta, &mut sequ).err_conv()?;
                    for dryrun in sequ.run_items() {
                        info!(target: ctx.path(), "regist undo {}", dryrun.gxl_meta().name());
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
                        //self.undo_transactions(ctx.clone(), undo_stack).await;
                        trans_manage.rollback(&ctx).await;
                        return Err(e);
                    }
                }
            }
        }

        Ok(TaskValue::from((def, ExecOut::Job(job))))
    }

    pub fn append_trans_hold<H: Into<TransableHold>>(&mut self, guard: &LableGuard, hold: H) {
        let trans_hold = hold.into();
        debug_assert!(trans_hold.assembled());
        debug!(target:"exec/sque", "only_item :{:?}", guard.lable());
        if !self.only_items().contains_key(guard.lable()) || *guard.open() {
            info!(target:"exec/sque", "only_item :{}", trans_hold.gxl_meta().full_name());
            self.only_items.insert(guard.lable().clone(), true);
            self.run_items.push(AsyncComHold::from(trans_hold).into());
        }
    }
}

impl AppendAble<AsyncComHold> for Sequence {
    fn append(&mut self, node: AsyncComHold) {
        debug!(target: "exec/sque", "append {}", node.gxl_meta().full_name() );
        self.run_items.push(node.into());
    }
}

impl AppendAble<IsolationHold> for Sequence {
    fn append(&mut self, node: IsolationHold) {
        debug!(target: "exec/sque", "append {}", node.hold().gxl_meta().full_name() );
        self.run_items.push(node.into());
    }
}

#[derive(Clone, Default)]
pub struct StubAction {
    name: String,
}

impl StubAction {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

impl ComponentMeta for StubAction {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from("stub")
    }
}

#[async_trait]
impl AsyncRunnableTrait for StubAction {
    async fn async_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        //TODO:
        debug!(target: ctx.path(), "executing stub: {}", self.name);
        let task = Task::from(&self.name);
        Ok(TaskValue::from((def, ExecOut::Task(task))))
    }
}
