pub trait StringCutter {
    fn cut_str(&self, max: usize) -> Self;
}

impl StringCutter for &str {
    fn cut_str(&self, max: usize) -> Self {
        if self.len() <= max {
            self
        } else {
            &self[..max]
        }
    }
}

impl StringCutter for String {
    fn cut_str(&self, max: usize) -> String {
        if self.len() <= max {
            self.clone()
        } else {
            self[..max].to_string()
        }
    }
}

#[derive(Default)]
pub struct UpperKeyMaker {
    prefix: Option<String>,
}
impl UpperKeyMaker {
    pub fn new<S: Into<String>>(prefix: S) -> Self {
        let mut value = prefix.into();
        let prefix_opt = if value.is_empty() {
            None
        } else {
            value.make_ascii_uppercase();
            Some(value)
        };
        Self { prefix: prefix_opt }
    }
    pub fn make<S: Into<String>>(&self, name: S) -> String {
        let mut key = name.into();
        key.make_ascii_uppercase();
        if let Some(prefix) = &self.prefix {
            format!("{}_{}", prefix, key)
        } else {
            key
        }
    }
}
