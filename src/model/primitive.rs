use derive_more::From;
use getset::{Getters, MutGetters, WithSetters};
use indexmap::IndexMap;

use super::sec::{SecFrom, SecValueType};

#[derive(Clone, Debug, MutGetters, Getters, WithSetters, PartialEq)]
#[getset(get = "pub")]
pub struct GxlFParam {
    name: String,
    #[getset(set_with = "pub", set = "pub")]
    default_name: bool,
    #[getset(set_with = "pub", set = "pub")]
    default_value: Option<SecValueType>,
}

impl GxlFParam {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            default_name: false,
            default_value: None,
        }
    }
}

#[derive(Clone, Debug, MutGetters, Getters, WithSetters, PartialEq)]
#[getset(get = "pub")]
pub struct GxlAParam {
    name: String,
    value: GxlObject,
}
impl GxlAParam {
    pub fn new<S: Into<String>>(name: S, value: GxlObject) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
    pub fn from_val<S: Into<String>>(name: S, val: S) -> Self {
        Self {
            name: name.into(),
            value: GxlObject::Value(SecValueType::nor_from(val.into())),
        }
    }
    pub fn from_ref<S: Into<String>>(name: S, val: S) -> Self {
        Self {
            name: name.into(),
            value: GxlObject::VarRef(val.into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq, PartialOrd, From)]
pub enum GxlObject {
    VarRef(String),
    Value(SecValueType),
}

impl GxlObject {
    pub fn from_val<S: Into<String>>(val: S) -> Self {
        Self::Value(SecValueType::nor_from(val.into()))
    }
    pub fn from_ref<S: Into<String>>(val: S) -> Self {
        Self::VarRef(val.into())
    }
}
pub type GxlFParams = Vec<GxlFParam>;
pub type GxlAParams = IndexMap<String, GxlAParam>;
