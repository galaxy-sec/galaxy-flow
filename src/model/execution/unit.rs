use crate::components::gxl_flow::meta::FlowMeta;
use crate::components::gxl_mod::meta::ModMeta;
use crate::meta::MetaInfo;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RunUnitLable {
    Entry(String),
    Exist(String),
    ModProp(String),
    Flow,
}
#[derive(Debug, Clone, Getters)]
pub struct RunUnitGuard {
    lable: RunUnitLable,
    open: bool,
}
impl Drop for RunUnitGuard {
    fn drop(&mut self) {
        self.open = false;
    }
}
impl RunUnitGuard {
    pub fn from_entry(value: &FlowMeta) -> Self {
        Self {
            lable: RunUnitLable::Entry(value.long_name()),
            open: true,
        }
    }

    pub fn from_exit(value: &FlowMeta) -> Self {
        Self {
            lable: RunUnitLable::Exist(value.long_name()),
            open: true,
        }
    }
    pub fn from_mod(value: &ModMeta) -> Self {
        Self {
            lable: RunUnitLable::ModProp(value.long_name()),
            open: true,
        }
    }
    pub fn from_flow() -> Self {
        Self {
            lable: RunUnitLable::Flow,
            open: true,
        }
    }
}
