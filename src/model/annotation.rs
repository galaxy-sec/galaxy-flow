use std::collections::HashMap;

use super::components::{
    gxl_env::anno::EnvAnnotation,
    gxl_flow::anno::{FlowAnnFunc, FlowAnnotation},
    gxl_mod::anno::ModAnnotation,
};

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
pub fn is_auto_func(ann: &FlowAnnotation, fn_name: &str) -> bool {
    ann.func == FlowAnnFunc::AutoLoad && ann.args.get(FST_ARG_TAG) == Some(&fn_name.to_string())
}
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

pub trait TaskMessage {
    fn message(&self) -> Option<String>;
}

pub trait ComUsage {
    fn desp(&self) -> Option<String>;
    fn color(&self) -> Option<String>;
}
