use crate::{
    annotation::{AnnTypeEnum, Annotation},
    data::FunDto,
};

#[derive(Clone, Debug, PartialEq)]
pub enum ModAnnFunc {
    GFlow,
    UnImpl,
}

impl From<&str> for ModAnnFunc {
    fn from(s: &str) -> Self {
        match s {
            "gflow" => ModAnnFunc::GFlow,
            _ => {
                warn!("UnImpl ModAnnFunc: {s}",);
                ModAnnFunc::UnImpl
            }
        }
    }
}

pub type ModAnnotation = Annotation<ModAnnFunc>;

impl From<FunDto> for ModAnnotation {
    fn from(dto: FunDto) -> ModAnnotation {
        let name = ModAnnFunc::from(dto.keyword.as_str());
        ModAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
        }
    }
}
