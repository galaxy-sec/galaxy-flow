use std::collections::HashMap;
pub type StrMap = HashMap<String, String>;

pub trait Mustable {
    fn must_get(&self, key: &str) -> &String;
}
impl Mustable for StrMap {
    fn must_get(&self, key: &str) -> &String {
        self.get(key)
            .unwrap_or_else(|| panic!("not get {} value", key))
    }
}

#[derive(Clone, Copy)]
pub enum LogicScope {
    Outer,
    Inner,
}

#[derive(Clone, Debug, Builder, PartialEq, Default)]
pub struct ShellOption {
    pub outer_print: bool,
    pub inner_print: bool,
    pub sudo: bool,
    pub err: Option<String>,
    pub suc: Option<String>,
    pub secrecy: bool,
    pub expect: Vec<i32>,
    pub log_lev: Option<log::Level>,
}

impl ShellOption {
    pub fn new() -> Self {
        ShellOption {
            outer_print: false,
            inner_print: false,
            sudo: false,
            err: None,
            suc: None,
            secrecy: false,
            expect: vec![0],
            log_lev: Some(log::Level::Info),
        }
    }
    pub fn new_explicit(outer: bool, inner: bool) -> Self {
        ShellOption {
            outer_print: outer,
            inner_print: inner,
            sudo: true,
            err: None,
            suc: None,
            secrecy: false,
            expect: vec![0],
            log_lev: Some(log::Level::Info),
        }
    }
    pub fn cmd_print(&self, scope: LogicScope) -> bool {
        match scope {
            LogicScope::Outer => self.outer_print,
            LogicScope::Inner => self.inner_print,
        }
    }
}

#[macro_export]
macro_rules! str_map {
    {} => ($crate::expect::StrMap::new());
    // In this implementation, key/value pairs separated by commas.
    { $( $key:expr => $value:expr ),* } => {
         str_map!( $( $key => $value, )* )
    };

    { $( $key:expr => $value:expr, )* } => ({
        let mut map = $crate::expect::StrMap::new();
        $(
            map.insert(String::from($key), String::from($value));
         )*
        map
     })
}
