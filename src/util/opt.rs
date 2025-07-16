use std::path::PathBuf;

pub trait OptionFrom<T> {
    fn to_opt(self) -> Option<T>;
}

impl<'a> OptionFrom<String> for &'a str {
    fn to_opt(self) -> Option<String> {
        Some(self.to_string())
    }
}

impl OptionFrom<PathBuf> for &str {
    fn to_opt(self) -> Option<PathBuf> {
        Some(PathBuf::from(self))
    }
}
