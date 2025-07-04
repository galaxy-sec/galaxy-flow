use core::str;
use std::fmt::Debug;

use orion_common::friendly::MultiNew2;

use crate::meta::{GxlType, MetaInfo};

use super::anno::ModAnnotation;

#[derive(Clone, Getters, Default)]
pub struct ModMeta {
    class: GxlType,
    name: String,
    mix: Vec<String>,
    annotations: Vec<ModAnnotation>,
}

impl Debug for ModMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("STCMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .finish()
    }
}
impl MetaInfo for ModMeta {
    fn full_name(&self) -> String {
        format!("[mod]:{}", self.name.clone())
    }
}
impl ModMeta {
    pub fn build_mod<S: Into<String>>(name: S) -> Self {
        ModMeta::new2(GxlType::Mod, name.into())
    }
}
impl MultiNew2<GxlType, String> for ModMeta {
    fn new2(cls: GxlType, name: String) -> Self {
        Self {
            class: cls,
            name,
            annotations: Vec::new(),
            mix: Vec::new(),
        }
    }
}
impl MultiNew2<GxlType, &str> for ModMeta {
    fn new2(cls: GxlType, name: &str) -> Self {
        Self {
            class: cls,
            name: name.into(),
            annotations: Vec::new(),
            mix: Vec::new(),
        }
    }
}

impl ModMeta {
    pub fn with_annotate(mut self, ann: ModAnnotation) -> Self {
        self.annotations.push(ann);
        self
    }
    pub fn with_annotates(mut self, anns: Vec<ModAnnotation>) -> Self {
        self.annotations = anns;
        self
    }
    pub fn add_annotate(&mut self, ann: ModAnnotation) {
        self.annotations.push(ann);
    }
    pub fn set_mix(&mut self, mix: Vec<String>) {
        self.mix = mix;
    }
}
