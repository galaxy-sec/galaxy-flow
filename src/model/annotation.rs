use std::collections::HashMap;

use super::components::gxl_flow::meta::FlowMetaHold;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Annotation<T> {
    pub name: String,
    pub args: HashMap<String, String>,
    pub ann_type: AnnTypeEnum,
    pub func: T,
}
pub const FST_ARG_TAG: &str = "_1";
#[allow(dead_code)]
pub const SEC_ARG_TAG: &str = "_2";
pub trait GetArgValue {
    fn get_arg(&self, key: &str) -> Option<String>;
}

impl<T> GetArgValue for Annotation<T> {
    fn get_arg(&self, key: &str) -> Option<String> {
        self.args.get(key).cloned()
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum AnnTypeEnum {
    #[default]
    Func,
}

pub trait TaskMessage {
    fn message(&self) -> Option<String>;
}

pub trait ComUsage {
    fn desp(&self) -> Option<String>;
    fn color(&self) -> Option<String>;
}

pub trait Transaction {
    fn is_transaction(&self) -> bool;
    fn undo_hold(&self) -> Option<FlowMetaHold>;
}

pub trait Dryrunable {
    fn dryrun_hold(&self) -> Option<FlowMetaHold>;
}
