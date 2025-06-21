use crate::{annotation::EnvAnnotation, meta::GxlType};

#[derive(Clone, Getters, Default)]
pub struct EnvMeta {
    class: GxlType,
    name: String,
    mix: Vec<String>,
    annotations: Vec<EnvAnnotation>,
}
