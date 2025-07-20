use std::sync::Arc;

use async_trait::async_trait;
use indexmap::IndexMap;
use orion_common::friendly::{AppendAble, New2};

use crate::{
    ability::prelude::{Action, TaskValue},
    meta::{GxlMeta, GxlType, MetaInfo},
    primitive::GxlValue,
    traits::PropsTrait,
};

use super::{
    gxl_var::GxlVar,
    prelude::{AsyncRunnableTrait, ComponentMeta, ExecContext, ExecOut, TaskResult, VarSpace},
};

pub trait MapKeyable {
    fn map_key(&self) -> String;
}
pub trait Vec2Mapable<T> {
    fn from_vec(vec: Vec<T>) -> Self;
    fn export_vec(&self) -> Vec<T>;
}

impl<T> Vec2Mapable<T> for IndexMap<String, T>
where
    T: MapKeyable + Clone,
{
    fn from_vec(value: Vec<T>) -> Self {
        let mut items: IndexMap<String, T> = IndexMap::new();
        value.into_iter().for_each(|x| {
            items.insert(x.map_key(), x);
        });
        items
    }

    fn export_vec(&self) -> Vec<T> {
        let mut data: Vec<T> = Vec::new();
        self.values().for_each(|x| data.push(x.clone()));
        data
    }
}

#[derive(Clone, Getters, PartialEq, Debug)]
pub struct PropMeta {
    class: GxlType,
    name: String,
}
impl MetaInfo for PropMeta {
    fn full_name(&self) -> String {
        format!("[props]:{}", self.name.clone())
    }
    fn long_name(&self) -> String {
        self.name.clone().to_string()
    }
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
    items: IndexMap<String, GxlVar>,
}
pub type VarsHold = Arc<GxlProps>;

impl From<Vec<GxlVar>> for GxlProps {
    fn from(value: Vec<GxlVar>) -> Self {
        Self {
            meta: PropMeta::new(""),
            host: "".to_string(),
            items: IndexMap::from_vec(value),
        }
    }
}
impl GxlProps {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            meta: PropMeta::new(name),
            ..Default::default()
        }
    }
    pub fn get<S: Into<String>>(&self, key: S) -> Option<&GxlVar> {
        self.items.get(&key.into())
    }
    pub fn mod_new<S: Into<String>>(name: S) -> GxlProps {
        let name_string = name.into();
        Self {
            meta: PropMeta::new(format!("{}.props", name_string.clone())),
            host: name_string,
            ..Default::default()
        }
    }
    pub fn with_vars(mut self, vars: Vec<GxlVar>) -> Self {
        self.items = IndexMap::from_vec(vars);
        self
    }
    pub fn insert<S: Into<String>>(&mut self, key: S, val: GxlValue) {
        let key_string = key.into();
        self.items
            .insert(key_string.clone(), GxlVar::new(key_string, val));
    }
    pub fn merge(&mut self, mut other: Self) {
        self.items.append(&mut other.items);
    }
    pub fn miss_merge(&mut self, other: Self) {
        for (k, v) in other.items {
            if !self.items.contains_key(&k) {
                self.items.insert(k, v);
            }
        }
    }
}
impl AppendAble<GxlVar> for GxlProps {
    fn append(&mut self, prop: GxlVar) {
        self.items.insert(prop.key().clone(), prop);
    }
}
impl AppendAble<Vec<GxlVar>> for GxlProps {
    fn append(&mut self, props: Vec<GxlVar>) {
        let mut target = IndexMap::from_vec(props);
        self.items.append(&mut target);
    }
}

impl PropsTrait for GxlProps {
    fn fetch_props(&self) -> Vec<GxlVar> {
        self.items.export_vec()
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlProps {
    async fn async_exec(&self, mut ctx: ExecContext, mut def: VarSpace) -> TaskResult {
        ctx.append("props");
        let action = Action::from("rg vars setting");
        self.export_props(ctx, def.global_mut(), self.host.as_str())?;
        Ok(TaskValue::from((def, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxlProps {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::from(self.meta().clone())
    }
}
