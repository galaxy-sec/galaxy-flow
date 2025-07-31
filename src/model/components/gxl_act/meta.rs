use getset::{Getters, WithSetters};

use crate::{
    components::gxl_mod::meta::ModMeta,
    meta::{GxlType, MetaInfo},
    primitive::GxlFParam,
};
use std::fmt::Debug;

#[derive(Clone, Getters, WithSetters, Default, PartialEq)]
#[getset(get = "pub")]
pub struct ActivityMeta {
    class: GxlType,
    name: String,
    #[getset(set_with = "pub")]
    params: Vec<GxlFParam>,
    host: Option<ModMeta>,
}

impl Debug for ActivityMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivityMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .field("params", &self.params)
            .finish()
    }
}

const UNKNOW: String = String::new();
impl MetaInfo for ActivityMeta {
    fn full_name(&self) -> String {
        let mod_name = self.host().as_ref().map_or(UNKNOW, |x| x.name().clone());
        format!("[activity]:{mod_name}.{}", self.name)
    }
    fn long_name(&self) -> String {
        let mod_name = self.host().as_ref().map_or(UNKNOW, |x| x.name().clone());
        format!("{mod_name}.{}", self.name)
    }
}

impl ActivityMeta {
    pub fn build<S: Into<String>>(name: S) -> Self {
        Self {
            class: GxlType::Activity,
            name: name.into(),
            ..Default::default()
        }
    }
}

impl ActivityMeta {
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
