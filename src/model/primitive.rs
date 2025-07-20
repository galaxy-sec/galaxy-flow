use getset::MutGetters;

use super::sec::{SecFrom, SecValueType};

#[derive(Clone, MutGetters, Getters)]
pub struct GxlArg {
    name: String,
    value: GxlValue,
}
impl GxlArg {
    pub fn new<S: Into<String>>(name: S, value: GxlValue) -> Self {
        Self {
            name: name.into(),
            value,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum GxlValue {
    VarRef(String),
    Value(SecValueType),
}

impl GxlValue {
    pub fn from_val<S: Into<String>>(val: S) -> Self {
        Self::Value(SecValueType::nor_from(val.into()))
    }
    pub fn from_ref<S: Into<String>>(val: S) -> Self {
        Self::VarRef(val.into())
    }
}
