use orion_common::friendly::New2;

use super::prelude::*;
use crate::ability::prelude::TaskValue;
use crate::execution::action::Action;
use crate::execution::runnable::ComponentMeta;
use crate::expect::StrMap;

use std::sync::Arc;

#[derive(Debug, Clone, Default, Getters, PartialEq)]
pub struct GxlProp {
    key: String,  //var_name;
    meta: String, //var_type;
    val: String,  //var_val ;
}

impl New2<String, String> for GxlProp {
    fn new(mut key: String, val: String) -> Self {
        key.make_ascii_uppercase();
        Self {
            key,
            meta: String::from("str"),
            val,
        }
    }
}
impl New2<&str, &str> for GxlProp {
    fn new(key: &str, val: &str) -> Self {
        Self::new(key.to_string(), val.to_string())
    }
}
impl GxlProp {
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
pub struct GxlVars {
    props: Vec<GxlProp>,
}
pub type VarsHold = Arc<GxlVars>;

impl GxlVars {
    pub fn insert<S: Into<String>>(&mut self, key: S, val: S) {
        self.props.push(GxlProp::new(key.into(), val.into()));
    }
    pub fn load(map: StrMap) -> Self {
        let mut obj = Self { props: Vec::new() };
        for (key, val) in map {
            obj.append(GxlProp::new(key, val));
        }
        obj
    }
}
impl AppendAble<GxlProp> for GxlVars {
    fn append(&mut self, prop: GxlProp) {
        self.props.push(prop);
    }
}

impl PropsTrait for GxlVars {
    fn fetch_props(&self) -> &Vec<GxlProp> {
        &self.props
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlVars {
    async fn async_exec(&self, ctx: ExecContext, mut def: VarSpace) -> VTResult {
        let action = Action::from("rg vars setting");
        self.export_props(ctx, def.global_mut(), "")?;
        Ok(TaskValue::from((def, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxlVars {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::from("vars")
    }
}
