use getset::MutGetters;

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GxlValue {
    VarRef(String),
    Value(String),
}

impl GxlValue {
    pub fn from_val<S: Into<String>>(val: S) -> Self {
        Self::Value(val.into())
    }
    pub fn from_ref<S: Into<String>>(val: S) -> Self {
        Self::VarRef(val.into())
    }
}
