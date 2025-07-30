use core::str;
use std::fmt::Debug;

use derive_more::From;

use super::components::{
    gxl_env::meta::EnvMeta, gxl_flow::meta::FlowMeta, gxl_fun::meta::FunMeta,
    gxl_mod::meta::ModMeta, gxl_prop::PropMeta,
};
#[derive(Debug, Clone, Default, PartialEq)]
pub enum GxlType {
    Env,
    Fun,
    Flow,
    Mod,
    Vars,
    Props,
    #[default]
    Ignore,
    Activity,
    Ability(String),
}

impl GxlType {
    pub fn ability(name: &str) -> Self {
        Self::Ability(name.to_string())
    }
}

#[derive(Clone, From)]
pub enum GxlMeta {
    Prop(PropMeta),
    Env(EnvMeta),
    Fun(FunMeta),
    Flow(FlowMeta),
    Mod(ModMeta),
    Simple(String),
}
pub trait MetaInfo {
    fn full_name(&self) -> String;
    fn long_name(&self) -> String;
}

impl GxlMeta {
    pub fn name(&self) -> &str {
        match self {
            GxlMeta::Prop(m) => m.name(),
            GxlMeta::Env(m) => m.name(),
            GxlMeta::Fun(m) => m.name(),
            GxlMeta::Flow(m) => m.name(),
            GxlMeta::Mod(m) => m.name(),
            GxlMeta::Simple(m) => m,
        }
    }
}
impl MetaInfo for GxlMeta {
    fn full_name(&self) -> String {
        match self {
            GxlMeta::Prop(m) => m.full_name(),
            GxlMeta::Env(m) => m.full_name(),
            GxlMeta::Fun(m) => m.full_name(),
            GxlMeta::Flow(m) => m.full_name(),
            GxlMeta::Mod(m) => m.full_name(),
            GxlMeta::Simple(m) => m.clone(),
        }
    }
    fn long_name(&self) -> String {
        match self {
            GxlMeta::Prop(m) => m.long_name(),
            GxlMeta::Env(m) => m.long_name(),
            GxlMeta::Fun(m) => m.long_name(),
            GxlMeta::Flow(m) => m.long_name(),
            GxlMeta::Mod(m) => m.long_name(),
            GxlMeta::Simple(m) => m.clone(),
        }
    }
}

impl From<&str> for GxlMeta {
    fn from(value: &str) -> Self {
        Self::Simple(value.to_string())
    }
}
