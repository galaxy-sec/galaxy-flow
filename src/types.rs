use std::sync::Arc;

pub type AnyResult<T> = anyhow::Result<T>;
pub type PairVec<T> = Vec<(T, T)>;

pub type Hold<T> = Arc<T>;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Property {
    pub key: String,
    pub val: String,
}

impl From<(String, String)> for Property {
    fn from(value: (String, String)) -> Self {
        Self {
            key: value.0,
            val: value.1,
        }
    }
}
impl From<(&str, &str)> for Property {
    fn from(value: (&str, &str)) -> Self {
        Self {
            key: value.0.to_string(),
            val: value.1.to_string(),
        }
    }
}
