use orion_common::friendly::New2;

use crate::primitive::GxlValue;

use super::{gxl_prop::MapKeyable, prelude::*};

#[derive(Debug, Clone, Getters, PartialEq)]
pub struct GxlVar {
    key: String,   //var_name;
    meta: String,  //var_type;
    val: GxlValue, //var_val ;
}

impl New2<String, GxlValue> for GxlVar {
    fn new(key: String, val: GxlValue) -> Self {
        //key.make_ascii_uppercase();
        Self {
            key,
            meta: String::from("str"),
            val,
        }
    }
}
impl New2<&str, GxlValue> for GxlVar {
    fn new(key: &str, val: GxlValue) -> Self {
        //key.make_ascii_uppercase();
        Self {
            key: key.into(),
            meta: String::from("str"),
            val,
        }
    }
}
impl New2<String, String> for GxlVar {
    fn new(key: String, val: String) -> Self {
        //key.make_ascii_uppercase();
        Self {
            key,
            meta: String::from("str"),
            val: GxlValue::from_val(val),
        }
    }
}
impl New2<&str, &str> for GxlVar {
    fn new(key: &str, val: &str) -> Self {
        Self::new(key.to_string(), val.to_string())
    }
}
impl GxlVar {
    pub fn ext_new(key: String, vtype: String, val: GxlValue) -> Self {
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

impl MapKeyable for GxlVar {
    fn map_key(&self) -> String {
        self.key.clone()
    }
}
