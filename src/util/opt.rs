use std::path::PathBuf;

use orion_sec::sec::{SecFrom, SecValueType};

pub trait OptionFrom<T> {
    fn to_opt(self) -> Option<T>;
}

impl OptionFrom<String> for &str {
    fn to_opt(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl OptionFrom<PathBuf> for &str {
    fn to_opt(self) -> Option<PathBuf> {
        Some(PathBuf::from(self))
    }
}

impl OptionFrom<SecValueType> for SecValueType {
    fn to_opt(self) -> Option<SecValueType> {
        Some(self)
    }
}

impl OptionFrom<SecValueType> for u64 {
    fn to_opt(self) -> Option<SecValueType> {
        Some(SecValueType::nor_from(self))
    }
}
