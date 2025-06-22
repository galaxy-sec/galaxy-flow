use orion_common::friendly::MultiNew2;

use crate::{annotation::ComUsage, meta::GxlType};
use std::fmt::Debug;

use super::anno::{FlowAnnotation, TransLable};
#[derive(Clone, Getters, Default)]
pub struct FlowMeta {
    class: GxlType,
    name: String,
    annotations: Vec<FlowAnnotation>,
    preorder: Vec<String>,
    postorder: Vec<String>,
}

impl Debug for FlowMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("STCMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .finish()
    }
}
impl FlowMeta {
    pub fn build_flow<S: Into<String>>(name: S) -> Self {
        Self {
            class: GxlType::Flow,
            name: name.into(),
            ..Default::default()
        }
    }
    pub fn build_flow_pre<S: Into<String>>(name: S, preorder: S) -> Self {
        Self {
            class: GxlType::Flow,
            name: name.into(),
            preorder: vec![preorder.into()],
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
    pub fn undo_flow_name(&self) -> Option<String> {
        for ann in &self.annotations {
            if ann.undo_flow_name().is_some() {
                return ann.undo_flow_name();
            }
        }
        None
    }
}
impl MultiNew2<GxlType, String> for FlowMeta {
    fn new2(cls: GxlType, name: String) -> Self {
        Self {
            class: cls,
            name,
            annotations: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
        }
    }
}
impl MultiNew2<GxlType, &str> for FlowMeta {
    fn new2(cls: GxlType, name: &str) -> Self {
        Self {
            class: cls,
            name: name.into(),
            annotations: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
        }
    }
}

impl FlowMeta {
    pub fn with_annotate(mut self, ann: FlowAnnotation) -> Self {
        self.annotations.push(ann);
        self
    }
    pub fn with_annotates(mut self, anns: Vec<FlowAnnotation>) -> Self {
        self.annotations = anns;
        self
    }
    pub fn set_annotates(&mut self, anns: Vec<FlowAnnotation>) {
        self.annotations = anns;
    }
    pub fn add_annotate(&mut self, ann: FlowAnnotation) {
        self.annotations.push(ann);
    }
    pub fn set_preorder<S: Into<String>>(&mut self, order: Vec<S>) {
        let mut preorder = Vec::new();
        for o in order {
            preorder.push(o.into())
        }
        self.preorder = preorder;
    }
    pub fn set_postorder<S: Into<String>>(&mut self, order: Vec<S>) {
        let mut postorder = Vec::new();
        for o in order {
            postorder.push(o.into())
        }
        self.postorder = postorder;
    }
}
