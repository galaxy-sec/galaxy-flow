use orion_common::friendly::New2;

use super::prelude::*;
use crate::execution::runnable::{ComponentMeta, VarSpace};
use crate::execution::task::Task;
use crate::expect::StrMap;

use std::sync::Arc;

#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct RgProp {
    key: String,  //var_name;
    meta: String, //var_type;
    val: String,  //var_val ;
}

impl New2<String, String> for RgProp {
    fn new(mut key: String, val: String) -> Self {
        key.make_ascii_uppercase();
        Self {
            key,
            meta: String::from("str"),
            val,
        }
    }
}
impl New2<&str, &str> for RgProp {
    fn new(key: &str, val: &str) -> Self {
        Self::new(key.to_string(), val.to_string())
    }
}
impl RgProp {
    pub fn ext_new(key: String, vtype: String, val: String) -> Self {
        Self {
            key,
            meta: vtype,
            val,
        }
    }
    pub fn set_prefix(&mut self, prefix: &str) {
        self.key = format!("{}_{}", prefix, self.key());
    }
}

#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct RgVars {
    props: Vec<RgProp>,
}
pub type VarsHold = Arc<RgVars>;

impl RgVars {
    pub fn insert<S: Into<String>>(&mut self, key: S, val: S) {
        self.props.push(RgProp::new(key.into(), val.into()));
    }
    pub fn load(map: StrMap) -> Self {
        let mut obj = Self { props: Vec::new() };
        for (key, val) in map {
            obj.append(RgProp::new(key, val));
        }
        obj
    }
}
impl AppendAble<RgProp> for RgVars {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}

impl PropsTrait for RgVars {
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

#[async_trait]
impl AsyncRunnableTrait for RgVars {
    async fn async_exec(&self, ctx: ExecContext, mut def: VarSpace) -> VTResult {
        let task = Task::from("rg vars setting");
        self.export_props(ctx, def.globle_mut(), "")?;
        Ok((def, ExecOut::Task(task)))
    }
}
impl ComponentMeta for RgVars {
    fn com_meta(&self) -> RgoMeta {
        RgoMeta::build_var("vars")
    }
}
