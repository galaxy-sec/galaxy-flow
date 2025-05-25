use async_trait::async_trait;
use derive_more::From;

use crate::ability::GxRead;
use crate::components::gxl_intercept::FlowRunner;
use crate::components::gxl_mod::ModRunner;
use crate::components::{GxlEnv, GxlFlow, GxlMod};
use crate::context::ExecContext;
use crate::meta::GxlMeta;

use super::runnable::{AsyncRunnableTrait, ComponentMeta, VTResult, VarSpace};
use super::sequence::RunStub;
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
pub struct IsolationHold {
    hold: AsyncComHold,
}

#[derive(Clone, From)]
pub enum ComHold {
    Conduction(AsyncComHold),
    Isolation(IsolationHold),
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

/*
#[derive(Clone, From)]
pub enum SyncComHold {
    Read(GxRead),
    Env(GxlEnv),
    Mox(GxlMod),
    Stub(RunStub),
    Runner(ModRunner),
}

#[derive(Clone, From)]
pub enum ComHold {
    AsyncCom(AsyncComHold),
    SyncCom(SyncComHold),
}

impl RunnableTrait for SyncComHold {
    fn exec(&self, ctx: ExecContext, dict: VarsDict) -> VTResult {
        match self {
            SyncComHold::Read(obj) => obj.exec(ctx, dict),
            SyncComHold::Env(obj) => obj.exec(ctx, dict),
            SyncComHold::Mox(obj) => obj.exec(ctx, dict),
            SyncComHold::Stub(obj) => obj.exec(ctx, dict),
            SyncComHold::Runner(obj) => obj.exec(ctx, dict),
        }
    }
}
impl ComponentMeta for SyncComHold {
    fn com_meta(&self) -> RgoMeta {
        match self {
            SyncComHold::Read(obj) => obj.com_meta(),
            SyncComHold::Env(obj) => obj.com_meta(),
            SyncComHold::Mox(obj) => obj.com_meta(),
            SyncComHold::Stub(obj) => obj.com_meta(),
            SyncComHold::Runner(obj) => obj.com_meta(),
        }
    }
}
*/
