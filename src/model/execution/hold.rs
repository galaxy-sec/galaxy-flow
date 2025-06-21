use async_trait::async_trait;
use derive_more::From;

use crate::ability::GxRead;
use crate::annotation::Transaction;
use crate::components::gxl_flow::runner::FlowRunner;
use crate::components::gxl_mod::body::ModRunner;
use crate::components::{GxlEnv, GxlFlow, GxlMod};
use crate::context::ExecContext;
use crate::meta::GxlMeta;

use super::runnable::{AsyncRunnableTrait, ComponentMeta, VTResult};
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
impl Transaction for AsyncComHold {
    fn is_transaction(&self) -> bool {
        match self {
            AsyncComHold::Flow(h) => h.is_transaction(),
            AsyncComHold::FlwRunner(h) => h.is_transaction(),
            AsyncComHold::Stub(h) => h.is_transaction(),
            AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => false,
        }
    }

    fn undo_flow(&self) -> Option<crate::annotation::FlowHold> {
        match self {
            AsyncComHold::Flow(h) => h.undo_flow(),
            AsyncComHold::FlwRunner(h) => h.undo_flow(),
            AsyncComHold::Stub(h) => h.undo_flow(),
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

    fn undo_flow(&self) -> Option<crate::annotation::FlowHold> {
        match self {
            ComHold::Conduction(h) => h.undo_flow(),
            ComHold::Isolation(h) => h.undo_flow(),
        }
    }
}
impl Transaction for IsolationHold {
    fn is_transaction(&self) -> bool {
        self.hold.is_transaction()
    }

    fn undo_flow(&self) -> Option<crate::annotation::FlowHold> {
        self.hold.undo_flow()
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
impl AsyncRunnableTrait for IsolationHold {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        let origin = dict.clone();
        let (_, task) = self.hold.async_exec(ctx, dict).await?;
        Ok((origin, task))
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
