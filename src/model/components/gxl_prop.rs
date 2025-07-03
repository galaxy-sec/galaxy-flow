use std::sync::Arc;

use async_trait::async_trait;
use orion_common::friendly::{AppendAble, New2};

use crate::{
    ability::prelude::{Action, TaskValue},
    meta::{GxlMeta, GxlType},
    traits::PropsTrait,
};

use super::{
    gxl_var::GxlVar,
    prelude::{AsyncRunnableTrait, ComponentMeta, ExecContext, ExecOut, TaskResult, VarSpace},
};

#[derive(Clone, Getters, PartialEq, Debug)]
pub struct PropMeta {
    class: GxlType,
    name: String,
}
impl PropMeta {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            class: GxlType::Props,
            name: name.into(),
        }
    }
}
impl Default for PropMeta {
    fn default() -> Self {
        Self {
            class: GxlType::Props,
            name: "".to_string(),
        }
    }
}

#[derive(Debug, Clone, Getters, Default, PartialEq)]
pub struct GxlProps {
    meta: PropMeta,
    host: String,
    items: Vec<GxlVar>,
}
pub type VarsHold = Arc<GxlProps>;

impl From<Vec<GxlVar>> for GxlProps {
    fn from(value: Vec<GxlVar>) -> Self {
        Self {
            meta: PropMeta::new(""),
            host: "".to_string(),
            items: value,
        }
    }
}
impl GxlProps {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            meta: PropMeta::new(name),
            host: "".to_string(),
            items: Vec::new(),
        }
    }
    pub fn mod_new<S: Into<String>>(name: S) -> GxlProps {
        let name_string = name.into();
        Self {
            meta: PropMeta::new(format!("{}.props", name_string.clone())),
            host: name_string,
            items: Vec::new(),
        }
    }
    pub fn with_vars(mut self, vars: Vec<GxlVar>) -> Self {
        self.items = vars;
        self
    }
    pub fn insert<S: Into<String>>(&mut self, key: S, val: S) {
        self.items.push(GxlVar::new(key.into(), val.into()));
    }
    /*
    pub fn load(map: StrMap) -> Self {
        let mut obj = Self { items: Vec::new() };
        for (key, val) in map {
            obj.append(GxlVar::new(key, val));
        }
        obj
    }
    */
    pub fn merge(&mut self, other: &Self) {
        self.items.append(&mut other.items().clone());
    }
}
impl AppendAble<GxlVar> for GxlProps {
    fn append(&mut self, prop: GxlVar) {
        self.items.push(prop);
    }
}

impl PropsTrait for GxlProps {
    fn fetch_props(&self) -> &Vec<GxlVar> {
        &self.items
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlProps {
    async fn async_exec(&self, ctx: ExecContext, mut def: VarSpace) -> TaskResult {
        let action = Action::from("rg vars setting");
        self.export_props(ctx, def.global_mut(), self.host.as_str())?;
        Ok(TaskValue::from((def, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxlProps {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from(self.meta().clone())
    }
}
