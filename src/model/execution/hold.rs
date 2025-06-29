use std::sync::Arc;

use async_trait::async_trait;
use derive_more::From;

use crate::ability::GxRead;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_flow::runner::FlowRunner;
use crate::components::gxl_mod::body::ModRunner;
use crate::components::{GxlEnv, GxlFlow, GxlMod};
use crate::context::ExecContext;
use crate::meta::GxlMeta;

use super::runnable::{AsyncRunnableTrait, ComponentMeta, TaskValue, VTResult};
use super::sequence::RunStub;
use super::VarSpace;
#[derive(Clone, From)]
pub enum AsyncComHold {
    Flow(GxlFlow),
    Stub(RunStub),
    EnvRunner(ModRunner),
    FlwRunner(FlowRunner),
    Read(GxRead),
    Env(GxlEnv),
    Mox(GxlMod),
}

#[derive(Clone, From)]
pub enum TransableHold {
    Flow(Arc<GxlFlow>),
    Stub(Arc<RunStub>),
    FlwRunner(Arc<FlowRunner>),
}

impl Transaction for AsyncComHold {
    fn is_transaction(&self) -> bool {
        let trans = match self {
            AsyncComHold::Flow(h) => h.is_transaction(),
            AsyncComHold::FlwRunner(h) => h.is_transaction(),
            AsyncComHold::Stub(h) => h.is_transaction(),
            AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => false,
        };
        info!(target:"trans",
            "{} is transaction :{}", self.com_meta().name(), trans);
        trans
    }

    fn undo_hold(&self) -> Option<TransableHold> {
        match self {
            AsyncComHold::Flow(h) => h.undo_hold(),
            AsyncComHold::FlwRunner(h) => h.undo_hold(),
            AsyncComHold::Stub(h) => h.undo_hold(),
            AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => None,
        }
    }
}
#[derive(Clone, From)]
pub struct IsolationHold {
    hold: AsyncComHold,
}

#[derive(Clone, From)]
pub enum ComHold {
    Conduction(AsyncComHold),
    Isolation(IsolationHold),
}
impl Transaction for ComHold {
    fn is_transaction(&self) -> bool {
        match self {
            ComHold::Conduction(h) => h.is_transaction(),
            ComHold::Isolation(h) => h.is_transaction(),
        }
    }

    fn undo_hold(&self) -> Option<TransableHold> {
        match self {
            ComHold::Conduction(h) => h.undo_hold(),
            ComHold::Isolation(h) => h.undo_hold(),
        }
    }
}
impl Dryrunable for ComHold {
    fn dryrun_hold(&self) -> Option<TransableHold> {
        match self {
            ComHold::Conduction(h) => h.dryrun_hold(),
            ComHold::Isolation(h) => h.dryrun_hold(),
        }
    }
}
impl Transaction for IsolationHold {
    fn is_transaction(&self) -> bool {
        self.hold.is_transaction()
    }

    fn undo_hold(&self) -> Option<TransableHold> {
        self.hold.undo_hold()
    }
}
impl Dryrunable for IsolationHold {
    fn dryrun_hold(&self) -> Option<TransableHold> {
        self.hold.dryrun_hold()
    }
}

impl Dryrunable for AsyncComHold {
    fn dryrun_hold(&self) -> Option<TransableHold> {
        match self {
            AsyncComHold::Flow(h) => h.dryrun_hold(),
            AsyncComHold::FlwRunner(h) => h.dryrun_hold(),
            AsyncComHold::Stub(_)
            | AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => None,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for AsyncComHold {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> VTResult {
        match self {
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
            Self::Stub(obj) => obj.async_exec(ctx, dct).await,
            Self::EnvRunner(obj) => obj.async_exec(ctx, dct).await,
            Self::FlwRunner(obj) => obj.async_exec(ctx, dct).await,
            Self::Read(obj) => obj.async_exec(ctx, dct).await,
            Self::Env(obj) => obj.async_exec(ctx, dct).await,
            Self::Mox(obj) => obj.async_exec(ctx, dct).await,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for TransableHold {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> VTResult {
        match self {
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
            Self::Stub(obj) => obj.async_exec(ctx, dct).await,
            Self::FlwRunner(obj) => obj.async_exec(ctx, dct).await,
        }
    }
}

impl ComponentMeta for TransableHold {
    fn com_meta(&self) -> GxlMeta {
        match self {
            TransableHold::Flow(h) => h.com_meta(),
            TransableHold::Stub(h) => h.com_meta(),
            TransableHold::FlwRunner(h) => h.flow().com_meta(),
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for IsolationHold {
    ///varspace isolation
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        let origin = dict.clone();
        let TaskValue { rec, .. } = self.hold.async_exec(ctx, dict).await?;
        Ok(TaskValue::from((origin, rec)))
    }
}

#[async_trait]
impl AsyncRunnableTrait for ComHold {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        match self {
            ComHold::Conduction(h) => h.async_exec(ctx, dict).await,
            ComHold::Isolation(h) => h.async_exec(ctx, dict).await,
        }
    }
}

impl ComponentMeta for ComHold {
    fn com_meta(&self) -> GxlMeta {
        match self {
            ComHold::Conduction(h) => h.com_meta(),
            ComHold::Isolation(h) => h.hold.com_meta(),
        }
    }
}

impl ComponentMeta for AsyncComHold {
    fn com_meta(&self) -> GxlMeta {
        match self {
            Self::Flow(obj) => obj.com_meta(),
            Self::Stub(obj) => obj.com_meta(),
            Self::EnvRunner(obj) => obj.com_meta(),
            Self::FlwRunner(obj) => obj.com_meta(),
            Self::Read(obj) => obj.com_meta(),
            Self::Env(obj) => obj.com_meta(),
            Self::Mox(obj) => obj.com_meta(),
        }
    }
}
