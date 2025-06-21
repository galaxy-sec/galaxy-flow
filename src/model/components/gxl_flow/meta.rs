use crate::{annotation::FlowAnnotation, meta::GxlType};

#[derive(Clone, Getters, Default)]
pub struct FlowMeta {
    class: GxlType,
    name: String,
    annotations: Vec<FlowAnnotation>,
    preorder: Vec<String>,
    postorder: Vec<String>,
}
