use std::sync::Arc;

use async_trait::async_trait;
use derive_more::From;

use crate::ability::GxRead;
use crate::annotation::{Dryrunable, Transaction};
use crate::components::gxl_flow::meta::FlowMetaHold;
use crate::components::gxl_spc::GxlSpace;
use crate::components::{GxlEnv, GxlFlow, GxlMod, GxlProps};
use crate::context::ExecContext;
use crate::meta::GxlMeta;
use crate::traits::DependTrait;

use super::runnable::{AsyncRunnableTrait, ComponentMeta, TaskResult, TaskValue};
use super::VarSpace;
#[derive(Clone, From)]
pub enum AsyncComHold {
    #[from(GxlFlow)]
    Flow(Arc<GxlFlow>),
    #[from(GxRead)]
    Read(Arc<GxRead>),
    #[from(GxlEnv)]
    Env(Arc<GxlEnv>),
    #[from(GxlMod)]
    Mox(Arc<GxlMod>),
    #[from(GxlProps)]
    Props(Arc<GxlProps>),
}

#[derive(Clone, From)]
pub enum TransableHold {
    #[from(GxlProps)]
    Props(Arc<GxlProps>),
    #[from(GxlFlow)]
    Flow(Arc<GxlFlow>),
}
impl From<TransableHold> for AsyncComHold {
    fn from(value: TransableHold) -> Self {
        match value {
            TransableHold::Props(o) => AsyncComHold::Props(o),
            TransableHold::Flow(o) => AsyncComHold::Flow(o),
        }
    }
}
impl TransableHold {
    pub fn assembled(&self) -> bool {
        match self {
            TransableHold::Props(_) => true,
            TransableHold::Flow(o) => *o.assembled(),
        }
    }
}
impl DependTrait<&GxlSpace> for TransableHold {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> crate::error::AResult<Self> {
        let obj = match self {
            TransableHold::Props(o) => Self::Props(o.clone()),
            TransableHold::Flow(o) => {
                Self::from(<GxlFlow as Clone>::clone(&o).assemble(mod_name, src)?)
            }
        };
        Ok(obj)
    }
}
impl Dryrunable for TransableHold {
    fn dryrun_hold(&self) -> Option<FlowMetaHold> {
        match self {
            TransableHold::Props(_) => None,
            TransableHold::Flow(o) => o.dryrun_hold(),
        }
    }
}

impl Transaction for AsyncComHold {
    fn is_transaction(&self) -> bool {
        let trans = match self {
            AsyncComHold::Flow(h) => h.is_transaction(),
            AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Props(_)
            | AsyncComHold::Mox(_) => false,
        };
        info!(target:"trans",
            "{} is transaction :{}", self.gxl_meta().name(), trans);
        trans
    }

    fn undo_hold(&self) -> Option<FlowMetaHold> {
        match self {
            AsyncComHold::Flow(h) => h.undo_hold(),
            AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Props(_)
            | AsyncComHold::Mox(_) => None,
        }
    }
}
#[derive(Clone, From, Getters)]
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

    fn undo_hold(&self) -> Option<FlowMetaHold> {
        match self {
            ComHold::Conduction(h) => h.undo_hold(),
            ComHold::Isolation(h) => h.undo_hold(),
        }
    }
}
impl Dryrunable for ComHold {
    fn dryrun_hold(&self) -> Option<FlowMetaHold> {
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

    fn undo_hold(&self) -> Option<FlowMetaHold> {
        self.hold.undo_hold()
    }
}
impl Dryrunable for IsolationHold {
    fn dryrun_hold(&self) -> Option<FlowMetaHold> {
        self.hold.dryrun_hold()
    }
}

impl Dryrunable for AsyncComHold {
    fn dryrun_hold(&self) -> Option<FlowMetaHold> {
        match self {
            AsyncComHold::Flow(h) => h.dryrun_hold(),
            AsyncComHold::Read(_)
            | AsyncComHold::Env(_)
            | AsyncComHold::Props(_)
            | AsyncComHold::Mox(_) => None,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for AsyncComHold {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        match self {
            Self::Props(obj) => obj.async_exec(ctx, dct).await,
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
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
            Self::Props(obj) => obj.async_exec(ctx, dct).await,
            Self::Flow(obj) => obj.async_exec(ctx, dct).await,
        }
    }
}

impl ComponentMeta for TransableHold {
    fn gxl_meta(&self) -> GxlMeta {
        match self {
            TransableHold::Props(h) => h.gxl_meta(),
            TransableHold::Flow(h) => h.gxl_meta(),
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
    fn gxl_meta(&self) -> GxlMeta {
        match self {
            ComHold::Conduction(h) => h.gxl_meta(),
            ComHold::Isolation(h) => h.hold.gxl_meta(),
        }
    }
}

impl ComponentMeta for AsyncComHold {
    fn gxl_meta(&self) -> GxlMeta {
        match self {
            Self::Props(obj) => obj.gxl_meta(),
            Self::Flow(obj) => obj.gxl_meta(),
            Self::Read(obj) => obj.gxl_meta(),
            Self::Env(obj) => obj.gxl_meta(),
            Self::Mox(obj) => obj.gxl_meta(),
        }
    }
}
