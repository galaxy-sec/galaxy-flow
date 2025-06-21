use orion_common::friendly::New3;

use crate::{
    annotation::{AnnTypeEnum, Annotation, ComUsage, GetArgValue, TaskMessage},
    parser::stc_base::FunDto,
    types::PairVec,
};

#[derive(Clone, Debug, PartialEq, Default)]
pub enum FlowAnnFunc {
    #[default]
    AutoLoad,
    Usage,
    UnImpl,
    Task,
    Dryrun,
}

impl From<&str> for FlowAnnFunc {
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

pub type FlowAnnotation = Annotation<FlowAnnFunc>;

impl TaskMessage for FlowAnnotation {
    fn message(&self) -> Option<String> {
        if self.func == FlowAnnFunc::Task {
            self.get_arg("name")
        } else {
            None
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

impl From<FunDto> for FlowAnnotation {
    fn from(dto: FunDto) -> FlowAnnotation {
        let name = FlowAnnFunc::from(dto.keyword.as_str());
        FlowAnnotation {
            name: dto.keyword.clone(),
            ann_type: AnnTypeEnum::Func,
            func: name,
            args: dto.args,
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
