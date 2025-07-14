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
#[derive(Clone)]
pub enum GxlValue {
    VarRef(String),
    Value(String),
}
