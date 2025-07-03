use std::sync::Arc;

use async_trait::async_trait;
use derive_more::From;

use crate::ability::GxRead;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_mod::body::ModRunner;
use crate::components::gxl_spc::GxlSpace;
use crate::components::{GxlEnv, GxlFlow, GxlMod};
use crate::context::ExecContext;
use crate::meta::GxlMeta;
use crate::traits::DependTrait;

use super::runnable::{AsyncRunnableTrait, ComponentMeta, TaskResult, TaskValue};
use super::sequence::RunStub;
use super::VarSpace;
#[derive(Clone, From)]
pub enum AsyncComHold {
    #[from(GxlFlow)]
    Flow(Arc<GxlFlow>),
    #[from(RunStub)]
    Stub(Arc<RunStub>),
    #[from(ModRunner)]
    EnvRunner(Arc<ModRunner>),
    #[from(GxRead)]
    Read(Arc<GxRead>),
    #[from(GxlEnv)]
    Env(Arc<GxlEnv>),
    #[from(GxlMod)]
    Mox(Arc<GxlMod>),
}

#[derive(Clone, From)]
pub enum TransableHold {
    #[from(GxlMod)]
    Mod(Arc<GxlMod>),
    #[from(GxlFlow)]
    Flow(Arc<GxlFlow>),
    #[from(RunStub)]
    Stub(Arc<RunStub>),
}
impl From<TransableHold> for AsyncComHold {
    fn from(value: TransableHold) -> Self {
        match value {
            TransableHold::Mod(o) => {
                debug_assert!(o.assembled(), "{} miss assemble ", o.meta().name());
                AsyncComHold::Mox(o)
            }
            TransableHold::Flow(o) => {
                debug_assert!(o.assembled());
                AsyncComHold::Flow(o)
            }
            TransableHold::Stub(o) => AsyncComHold::Stub(o),
        }
    }
}
/*
impl DependTrait<&GxlSpace> for TransableHold {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> crate::error::AResult<Self> {
        let obj = match self {
            TransableHold::Mod(o) => {
                //Self::Mod(o.clone()),
                Self::from(<GxlMod as Clone>::clone(&o).assemble(mod_name, src)?)
            }
            TransableHold::Flow(o) => {
                Self::from(<GxlFlow as Clone>::clone(&o).assemble(mod_name, src)?)
            }
            TransableHold::Stub(_o) => todo!(),
        };
        Ok(obj)
    }
}
*/

impl Transaction for AsyncComHold {
    fn is_transaction(&self) -> bool {
        let trans = match self {
            AsyncComHold::Flow(h) => h.is_transaction(),
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

    fn undo_hold(&self) -> Vec<TransableHold> {
        match self {
            AsyncComHold::Flow(h) => h.undo_hold(),
            AsyncComHold::Stub(h) => h.undo_hold(),
            AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => Vec::new(),
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

    fn undo_hold(&self) -> Vec<TransableHold> {
        match self {
            ComHold::Conduction(h) => h.undo_hold(),
            ComHold::Isolation(h) => h.undo_hold(),
        }
    }
}
impl Dryrunable for ComHold {
    fn dryrun_hold(&self) -> Vec<TransableHold> {
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

    fn undo_hold(&self) -> Vec<TransableHold> {
        self.hold.undo_hold()
    }
}
impl Dryrunable for IsolationHold {
    fn dryrun_hold(&self) -> Vec<TransableHold> {
        self.hold.dryrun_hold()
    }
}

impl Dryrunable for AsyncComHold {
    fn dryrun_hold(&self) -> Vec<TransableHold> {
        match self {
            AsyncComHold::Flow(h) => h.dryrun_hold(),
            AsyncComHold::Stub(_)
            | AsyncComHold::EnvRunner(_)
            | AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Mox(_) => Vec::new(),
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for AsyncComHold {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        match self {
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
            Self::Stub(obj) => obj.async_exec(ctx, dct).await,
            Self::EnvRunner(obj) => obj.async_exec(ctx, dct).await,
            Self::Read(obj) => obj.async_exec(ctx, dct).await,
            Self::Env(obj) => obj.async_exec(ctx, dct).await,
            Self::Mox(obj) => obj.async_exec(ctx, dct).await,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for TransableHold {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        match self {
            Self::Mod(obj) => obj.async_exec(ctx, dct).await,
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
            Self::Stub(obj) => obj.async_exec(ctx, dct).await,
        }
    }
}

impl ComponentMeta for TransableHold {
    fn com_meta(&self) -> GxlMeta {
        match self {
            TransableHold::Mod(h) => h.com_meta(),
            TransableHold::Flow(h) => h.com_meta(),
            TransableHold::Stub(h) => h.com_meta(),
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for IsolationHold {
    ///varspace isolation
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> TaskResult {
        let origin = dict.clone();
        let TaskValue { rec, .. } = self.hold.async_exec(ctx, dict).await?;
        Ok(TaskValue::from((origin, rec)))
    }
}

#[async_trait]
impl AsyncRunnableTrait for ComHold {
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> TaskResult {
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
            Self::Read(obj) => obj.com_meta(),
            Self::Env(obj) => obj.com_meta(),
            Self::Mox(obj) => obj.com_meta(),
        }
    }
}
