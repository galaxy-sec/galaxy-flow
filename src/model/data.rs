use std::collections::HashMap;
use std::fmt::Display;
#[derive(Default, Debug, Clone, PartialEq)]
pub struct FunDto {
    pub keyword: String,
    pub args: HashMap<String, String>,
}
impl Display for FunDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ann-fun : {}(", self.keyword)?;
        for (k, v) in &self.args {
            write!(f, "{}:{},", k, v)?;
        }
        write!(f, ")",)?;
        Ok(())
    }
}

impl FunDto {
    pub fn new(keyword: &str, args: Vec<(&str, &str)>) -> Self {
        Self {
            keyword: keyword.to_string(),
            args: args
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AnnDto {
    pub funs: Vec<FunDto>,
}
#[derive(Default, Debug, Clone)]
pub struct HeadDto {
    pub keyword: String,
    pub name: String,
    pub before: Vec<String>,
    pub after: Vec<String>,
}
