use contracts::requires;

use crate::{
    annotation::ComUsage,
    components::gxl_mod::meta::ModMeta,
    meta::{GxlType, MetaInfo},
};
use std::{fmt::Debug, sync::Arc};

use super::anno::{DryrunAnno, FlowAnnotation, TransAnno};
#[derive(Clone, Getters, Default)]
pub struct FlowMeta {
    class: GxlType,
    name: String,
    host: Option<ModMeta>,
    annotations: Vec<FlowAnnotation>,
    preorder: Vec<String>,
    postorder: Vec<String>,
    pre_metas: Vec<FlowMeta>,
    pos_metas: Vec<FlowMeta>,
    undo_meta: Option<Arc<FlowMeta>>,
    dryrun_meta: Option<Arc<FlowMeta>>,
}
pub type FlowMetaHold = Arc<FlowMeta>;

impl Debug for FlowMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FlowMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .finish()
    }
}

const UNKNOW: String = String::new();
impl MetaInfo for FlowMeta {
    #[requires(self.host.is_some())]
    fn full_name(&self) -> String {
        let mod_name = self
            .host()
            .as_ref()
            .map(|x| x.name().clone())
            .unwrap_or(UNKNOW);
        format!("[flow]:{mod_name}.{}", self.name)
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
    #[requires(self.host.is_some())]
    pub fn undo_flow_name(&self) -> Option<String> {
        for ann in &self.annotations {
            if ann.undo_flow_name().is_some() {
                return ann.undo_flow_name();
            }
        }
        None
    }

    #[requires(self.host.is_some())]
    pub(crate) fn dryrun_flow_name(&self) -> Option<String> {
        for ann in &self.annotations {
            if ann.dryrun_flow_name().is_some() {
                return ann.dryrun_flow_name();
            }
        }
        None
    }
    pub fn pre_metas_mut(&mut self) -> &mut Vec<FlowMeta> {
        &mut self.pre_metas
    }
    pub fn pos_metas_mut(&mut self) -> &mut Vec<FlowMeta> {
        &mut self.pos_metas
    }

    pub fn set_undo(&mut self, undo: FlowMeta) {
        self.undo_meta.replace(Arc::new(undo));
    }
    pub fn set_dryrun(&mut self, dryrun: FlowMeta) {
        self.dryrun_meta.replace(Arc::new(dryrun));
    }
}

impl FlowMeta {
    pub fn new<S: Into<String>>(cls: GxlType, name: S) -> Self {
        Self {
            class: cls,
            name: name.into(),
            ..Default::default()
        }
    }
    pub fn with_annotate(mut self, ann: FlowAnnotation) -> Self {
        self.annotations.push(ann);
        self
    }
    pub fn with_annotates(mut self, anns: Vec<FlowAnnotation>) -> Self {
        self.annotations = anns;
        self
    }
    pub fn set_host(&mut self, mod_meta: ModMeta) {
        self.host = Some(mod_meta);
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
