use contracts::requires;
use getset::{Getters, WithSetters};

use crate::{
    components::gxl_mod::meta::ModMeta,
    meta::{GxlType, MetaInfo},
    primitive::GxlFParam,
};
use std::{fmt::Debug, sync::Arc};

#[derive(Clone, Getters, Default, WithSetters)]
#[getset(get = "pub")]
pub struct FunMeta {
    class: GxlType,
    name: String,
    #[getset(set_with = "pub")]
    params: Vec<GxlFParam>,
    host: Option<ModMeta>,
}
pub type FlowMetaHold = Arc<FunMeta>;

impl Debug for FunMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FunMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .field("params", &self.params)
            .finish()
    }
}

const UNKNOW: String = String::new();
impl MetaInfo for FunMeta {
    #[requires(self.host.is_some())]
    fn full_name(&self) -> String {
        let mod_name = self
            .host()
            .as_ref()
            .map(|x| x.name().clone())
            .unwrap_or(UNKNOW);
        format!("[fun]:{mod_name}.{}", self.name)
    }
    #[requires(self.host.is_some())]
    fn long_name(&self) -> String {
        let mod_name = self
            .host()
            .as_ref()
            .map(|x| x.name().clone())
            .unwrap_or(UNKNOW);
        format!("{mod_name}.{}", self.name)
    }
}

impl FunMeta {
    pub fn build_fun<S: Into<String>>(name: S) -> Self {
        Self {
            class: GxlType::Fun,
            name: name.into(),
            ..Default::default()
        }
    }
}

impl FunMeta {
    pub fn new<S: Into<String>>(cls: GxlType, name: S) -> Self {
        Self {
            class: cls,
            name: name.into(),
            ..Default::default()
        }
    }
    pub fn set_host(&mut self, mod_meta: ModMeta) {
        self.host = Some(mod_meta);
    }
}
