use std::{collections::HashMap, convert};

use orion_common::friendly::New3;

use crate::types::PairVec;

#[derive(Clone, Default, Debug, PartialEq)]
pub struct Annotation<T> {
    pub name: String,
    pub args: HashMap<String, String>,
    pub ann_type: AnnTypeEnum,
    pub func: T,
}
pub type FlowAnnotation = Annotation<FlowAnnFunc>;
pub type EnvAnnotation = Annotation<EnvAnnFunc>;
pub type ModAnnotation = Annotation<ModAnnFunc>;
pub const FST_ARG_TAG: &str = "_1";
#[allow(dead_code)]
pub const SEC_ARG_TAG: &str = "_2";
pub fn is_auto_func(ann: &AnnEnum, fn_name: &str) -> bool {
    match ann {
        AnnEnum::Flow(annotation) => {
            annotation.func == FlowAnnFunc::AutoLoad
                && annotation.args.get(FST_ARG_TAG) == Some(&fn_name.to_string())
        }
        AnnEnum::Mod(_) => false,
        AnnEnum::Env(_) => false,
    }
}
trait GetArgValue {
    fn get_arg(&self, key: &str) -> Option<String>;
}

impl<T> GetArgValue for Annotation<T> {
    fn get_arg(&self, key: &str) -> Option<String> {
        self.args.get(key).cloned()
    }
}

impl ComUsage for FlowAnnotation {
    fn desp(&self) -> Option<String> {
        if self.func == FlowAnnFunc::Usage {
            self.get_arg("desp")
        } else {
            None
        }
    }
    fn color(&self) -> Option<String> {
        if self.func == FlowAnnFunc::Usage {
            self.get_arg("color")
        } else {
            None
        }
    }
}

impl TaskMessage for FlowAnnotation {
    fn message(&self) -> Option<String> {
        if self.func == FlowAnnFunc::Task {
            self.get_arg("name")
        } else {
            None
        }
    }
}

impl ComUsage for EnvAnnotation {
    fn desp(&self) -> Option<String> {
        if self.func == EnvAnnFunc::Usage {
            self.get_arg("desp")
        } else {
            None
        }
    }

    fn color(&self) -> Option<String> {
        if self.func == EnvAnnFunc::Usage {
            self.get_arg("color")
        } else {
            None
        }
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum AnnTypeEnum {
    #[default]
    Func,
}
#[derive(Clone, Debug, PartialEq)]
pub enum AnnEnum {
    Flow(FlowAnnotation),
    Mod(ModAnnotation),
    Env(EnvAnnotation),
}

pub trait Autoload {
    fn is_autoload(&self) -> (bool, Vec<ModAop>);
}

pub enum ModAop {
    Entry,
    Exit,
}
impl Autoload for Vec<AnnEnum> {
    fn is_autoload(&self) -> (bool, Vec<ModAop>) {
        let mut aop = Vec::new();
        let mut auto = false;
        for ann in self {
            let (a, mut b) = ann.is_autoload();
            auto = auto || a;
            aop.append(&mut b);
        }
        (auto, aop)
    }
}
impl Autoload for AnnEnum {
    fn is_autoload(&self) -> (bool, Vec<ModAop>) {
        let mut aop = Vec::new();
        let auto = match self {
            AnnEnum::Flow(f) => {
                if f.func == FlowAnnFunc::AutoLoad {
                    for k in f.args.keys() {
                        match k.as_str() {
                            "entry" => aop.push(ModAop::Entry),
                            "exit" => aop.push(ModAop::Exit),
                            _ => {}
                        }
                    }
                    true
                } else {
                    false
                }
            }
            AnnEnum::Mod(_) => false,
            AnnEnum::Env(_) => false,
        };
        (auto, aop)
    }
}

impl From<FlowAnnotation> for AnnEnum {
    fn from(f: FlowAnnotation) -> Self {
        AnnEnum::Flow(f)
    }
}
impl From<ModAnnotation> for AnnEnum {
    fn from(f: ModAnnotation) -> Self {
        AnnEnum::Mod(f)
    }
}
impl From<EnvAnnotation> for AnnEnum {
    fn from(f: EnvAnnotation) -> Self {
        AnnEnum::Env(f)
    }
}

#[derive(Clone, Debug, PartialEq, Default)]
pub enum FlowAnnFunc {
    #[default]
    AutoLoad,
    Usage,
    UnImpl,
    Task,
    Dryrun,
}
#[derive(Clone, Debug, PartialEq)]
pub enum EnvAnnFunc {
    Usage,
    UnImpl,
}

pub trait TaskMessage {
    fn message(&self) -> Option<String>;
}

pub trait ComUsage {
    fn desp(&self) -> Option<String>;
    fn color(&self) -> Option<String>;
}

#[derive(Clone, Debug, PartialEq)]
pub enum ModAnnFunc {
    GFlow,
    UnImpl,
}
impl convert::From<&str> for FlowAnnFunc {
    fn from(s: &str) -> Self {
        match s {
            "auto_load" => FlowAnnFunc::AutoLoad,
            "usage" => FlowAnnFunc::Usage,
            "task" => FlowAnnFunc::Task,
            "dryrun" => FlowAnnFunc::Dryrun,
            _ => {
                warn!("UnImpl FlowAnnFunc: {}", s);
                FlowAnnFunc::UnImpl
            }
        }
    }
}
impl convert::From<&str> for EnvAnnFunc {
    fn from(s: &str) -> Self {
        match s {
            "usage" => EnvAnnFunc::Usage,
            _ => {
                warn!("UnImpl FlowAnnFunc: {}", s);
                EnvAnnFunc::UnImpl
            }
        }
    }
}

impl convert::From<&str> for ModAnnFunc {
    fn from(s: &str) -> Self {
        match s {
            "gflow" => ModAnnFunc::GFlow,
            _ => {
                warn!("UnImpl ModAnnFunc: {}", s);
                ModAnnFunc::UnImpl
            }
        }
    }
}

impl From<(&str, PairVec<&str>)> for FlowAnnotation {
    fn from(value: (&str, PairVec<&str>)) -> Self {
        Self {
            name: value.0.to_string(),
            ann_type: AnnTypeEnum::Func,
            func: FlowAnnFunc::UnImpl,
            args: value
                .1
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}

impl New3<FlowAnnFunc, &str, Vec<(&str, &str)>> for FlowAnnotation {
    fn new(func: FlowAnnFunc, name: &str, args: Vec<(&str, &str)>) -> Self {
        Self {
            name: name.to_string(),
            ann_type: AnnTypeEnum::Func,
            func,
            args: args
                .into_iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
        }
    }
}
impl New3<FlowAnnFunc, String, Vec<(String, String)>> for FlowAnnotation {
    fn new(func: FlowAnnFunc, name: String, args: Vec<(String, String)>) -> Self {
        Self {
            name,
            ann_type: AnnTypeEnum::Func,
            func,
            args: args.into_iter().collect(),
        }
    }
}

impl New3<EnvAnnFunc, String, Vec<(String, String)>> for EnvAnnotation {
    fn new(func: EnvAnnFunc, name: String, args: Vec<(String, String)>) -> Self {
        Self {
            name,
            ann_type: AnnTypeEnum::Func,
            func,
            args: args.into_iter().collect(),
        }
    }
}

/*
impl New1<String> for FlowAnnota {
    type Ins = Self;
    fn new(name: String) -> Self {
        Self {
            name: name,
            ann_type: AnnTypeEnum::FUNC,
            func: AnnAllowFunc::Load,
            args: Vec::new(),
        }
    }
}
*/

#[cfg(test)]
mod tests {

    use super::*;
    #[test]
    fn test_flowannota_new() {
        let a = FlowAnnotation::new(
            FlowAnnFunc::AutoLoad,
            "test".to_string(),
            vec![("a".to_string(), "b".to_string())],
        );
        assert_eq!(a.name, "test");
        assert_eq!(a.args.len(), 1);
        assert_eq!(a.args["a"], "b");

        let a = FlowAnnotation::new(FlowAnnFunc::AutoLoad, "test", vec![("a", "b")]);
        //let a = FlowAnnota::new("test", vec![("a", "b")]);
        assert_eq!(a.name, "test");
        assert_eq!(a.args.len(), 1);
        assert_eq!(a.args["a"], "b");
    }
    // test flowannota defalut_new
    #[test]
    fn test_flowannota_default_new() {
        let a = FlowAnnotation::from(("test", vec![("a", "b")]));
        assert_eq!(a.name, "test");
        assert_eq!(a.args.len(), 1);
        assert_eq!(a.args["a"], "b");
    }
}
