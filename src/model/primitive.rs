use derive_more::From;
use getset::MutGetters;

use super::sec::{SecFrom, SecValueType};

#[derive(Clone, MutGetters, Getters)]
pub struct GxlArg {
    name: String,
    value: GxlObject,
}
impl GxlArg {
    pub fn new<S: Into<String>>(name: S, value: GxlObject) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}
#[derive(Clone, Debug, PartialEq, From)]
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
