use crate::{
    annotation::ComUsage,
    components::gxl_mod::meta::ModMeta,
    meta::{GxlType, MetaInfo},
};
use orion_common::friendly::MultiNew2;
use std::fmt::Debug;

use super::anno::EnvAnnotation;

#[derive(Clone, Getters, Default)]
pub struct EnvMeta {
    class: GxlType,
    name: String,
    host: Option<ModMeta>,
    mix: Vec<String>,
    annotations: Vec<EnvAnnotation>,
}

impl Debug for EnvMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("STCMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .finish()
    }
}
impl MetaInfo for EnvMeta {
    fn full_name(&self) -> String {
        format!("[env]:{}:{}", self.name, self.mix.join(","))
    }
}

impl EnvMeta {
    pub fn build_env<S: Into<String>>(name: S) -> Self {
        Self::new2(GxlType::Env, name.into())
    }
    pub fn build_env_mix<S: Into<String> + Clone>(name: S, mix: Vec<S>) -> Self {
        let mut mix_string: Vec<String> = Vec::new();
        mix.iter()
            .for_each(|i: &S| mix_string.push(i.clone().into()));
        Self {
            class: GxlType::Env,
            name: name.into(),
            mix: mix_string,
            ..Default::default()
        }
    }
    pub fn desp(&self) -> Option<String> {
        for ann in &self.annotations {
            if ann.desp().is_some() {
                return ann.desp();
            }
        }
        None
    }
    pub fn color(&self) -> Option<String> {
        for ann in &self.annotations {
            if ann.color().is_some() {
                return ann.color();
            }
        }
        None
    }

    pub(crate) fn with_host(&self, mod_meta: crate::components::gxl_mod::meta::ModMeta) -> _ {
        todo!()
    }
}
impl MultiNew2<GxlType, String> for EnvMeta {
    fn new2(cls: GxlType, name: String) -> Self {
        Self {
            class: cls,
            name,
            ..Default::default()
        }
    }
}
impl MultiNew2<GxlType, &str> for EnvMeta {
    fn new2(cls: GxlType, name: &str) -> Self {
        Self {
            class: cls,
            name: name.into(),
            ..Default::default()
        }
    }
}

impl EnvMeta {
    pub fn with_annotate(mut self, ann: EnvAnnotation) -> Self {
        self.annotations.push(ann);
        self
    }
    pub fn with_annotates(mut self, anns: Vec<EnvAnnotation>) -> Self {
        self.annotations = anns;
        self
    }
    pub fn set_host(&mut self, host: ModMeta) {
        self.host = Some(host);
    }
    pub fn set_annotates(&mut self, anns: Vec<EnvAnnotation>) {
        self.annotations = anns;
    }
    pub fn add_annotate(&mut self, ann: EnvAnnotation) {
        self.annotations.push(ann);
    }
    pub fn set_mix(&mut self, mix: Vec<String>) {
        self.mix = mix;
    }
}
