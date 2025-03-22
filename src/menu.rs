#[derive(Default, Getters, Builder, Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub key: String,
    pub desp: Option<String>,
    pub color: Option<String>,
}
impl MenuItem {
    pub fn new(key: String, desp: Option<String>, color: Option<String>) -> Self {
        MenuItem { key, desp, color }
    }
}
#[derive(Default, Getters, Builder, Debug, PartialEq, Clone)]
pub struct GxMenu {
    pub envs: Vec<MenuItem>,
    pub flows: Vec<MenuItem>,
}
impl GxMenu {
    pub fn merge(&mut self, other: &mut GxMenu) {
        self.envs.append(&mut other.envs);
        self.flows.append(&mut other.flows);
    }
}
