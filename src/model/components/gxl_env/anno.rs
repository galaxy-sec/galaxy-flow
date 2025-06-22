use orion_common::friendly::New3;

use crate::{
    annotation::{AnnTypeEnum, Annotation, ComUsage, GetArgValue},
    data::FunDto,
};

#[derive(Clone, Debug, PartialEq)]
pub enum EnvAnnFunc {
    Usage,
    UnImpl,
}

impl From<&str> for EnvAnnFunc {
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

pub type EnvAnnotation = Annotation<EnvAnnFunc>;

impl From<FunDto> for EnvAnnotation {
    fn from(dto: FunDto) -> EnvAnnotation {
        let name = EnvAnnFunc::from(dto.keyword.as_str());
        EnvAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
        }
    }
}
