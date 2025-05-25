use core::str;
use std::fmt::Debug;

use orion_common::friendly::{MultiNew2, MultiNew3};

use crate::model::annotation::{AnnEnum, ComUsage};
#[derive(Debug, Clone, Default)]
pub enum GxlType {
    Env,
    Flow,
    Mod,
    Vars,
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

#[derive(Clone, Getters, Default)]
pub struct GxlMeta {
    class: GxlType,
    name: String,
    mix: Vec<String>,
    annotations: Vec<AnnEnum>,
    preorder: Vec<String>,
    postorder: Vec<String>,
}

impl Debug for GxlMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("STCMeta")
            .field("class", &self.class)
            .field("name", &self.name)
            .finish()
    }
}
impl GxlMeta {
    pub(crate) fn build_ability(arg: &str) -> GxlMeta {
        GxlMeta::new2(GxlType::Ability(arg.to_string()), arg.to_string())
    }
    pub fn build_activity(arg: &str) -> GxlMeta {
        GxlMeta::new2(GxlType::Activity, arg.to_string())
    }
    pub fn build_var(arg: &str) -> GxlMeta {
        GxlMeta::new2(GxlType::Vars, arg.to_string())
    }

    pub fn build_mod<S: Into<String>>(name: S) -> Self {
        GxlMeta::new2(GxlType::Mod, name.into())
    }
    pub fn build_env<S: Into<String>>(name: S) -> Self {
        GxlMeta::new2(GxlType::Env, name.into())
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
    pub fn set_anns(&mut self, anns: Vec<AnnEnum>) {
        self.annotations = anns;
    }
    pub fn desp(&self) -> Option<String> {
        for ann in &self.annotations {
            match ann {
                AnnEnum::Flow(ann) => {
                    if ann.desp().is_some() {
                        return ann.desp();
                    }
                }
                AnnEnum::Env(ann) => {
                    if ann.desp().is_some() {
                        return ann.desp();
                    }
                }
                _ => {}
            }
        }
        None
    }
    pub fn color(&self) -> Option<String> {
        for ann in &self.annotations {
            match ann {
                AnnEnum::Flow(ann) => {
                    if ann.color().is_some() {
                        return ann.color();
                    }
                }
                AnnEnum::Env(ann) => {
                    if ann.color().is_some() {
                        return ann.color();
                    }
                }
                _ => {}
            }
        }
        None
    }
}
impl MultiNew2<GxlType, String> for GxlMeta {
    fn new2(cls: GxlType, name: String) -> Self {
        Self {
            class: cls,
            name,
            annotations: Vec::new(),
            mix: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
        }
    }
}
impl MultiNew2<GxlType, &str> for GxlMeta {
    fn new2(cls: GxlType, name: &str) -> Self {
        Self {
            class: cls,
            name: name.into(),
            annotations: Vec::new(),
            mix: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
        }
    }
}

impl MultiNew3<GxlType, String, Vec<AnnEnum>> for GxlMeta {
    fn new3(cls: GxlType, name: String, anns: Vec<AnnEnum>) -> Self {
        Self {
            class: cls,
            name,
            annotations: anns,
            mix: Vec::new(),
            preorder: Vec::new(),
            postorder: Vec::new(),
        }
    }
}

impl GxlMeta {
    pub fn add_annotate(&mut self, ann: AnnEnum) {
        self.annotations.push(ann);
    }
    pub fn set_mix(&mut self, mix: Vec<String>) {
        self.mix = mix;
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
