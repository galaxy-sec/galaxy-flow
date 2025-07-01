use crate::ability::prelude::TaskValue;

use super::gxl_spc::GxlSpace;
use super::{prelude::*, GxlFlow};

use super::gxl_var::GxlProp;

#[derive(Clone, Getters)]
pub struct GxlIntercept {
    m_name: String,
    props: Vec<GxlProp>,
    flows: Vec<GxlFlow>,
}

impl GxlIntercept {
    pub fn new(m_name: String, props: Vec<GxlProp>, flows: Vec<GxlFlow>) -> Self {
        Self {
            m_name,
            props,
            flows,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlIntercept {
    async fn async_exec(&self, ctx: ExecContext, mut var_dict: VarSpace) -> TaskResult {
        let mut job = Job::from("intercept");
        self.export_props(ctx.clone(), var_dict.global_mut(), self.m_name())?;
        for flow in &self.flows {
            let TaskValue { vars, rec, .. } = flow.async_exec(ctx.clone(), var_dict).await?;
            var_dict = vars;
            job.append(rec);
        }
        Ok(TaskValue::from((var_dict, ExecOut::Job(job))))
    }
}
impl DependTrait<&GxlSpace> for GxlIntercept {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        let mut flows = Vec::new();
        for flow in self.flows {
            flows.push(flow.assemble(mod_name, src)?);
        }
        Ok(Self {
            m_name: self.m_name,
            props: self.props,
            flows,
        })
    }
}

impl PropsTrait for GxlIntercept {
    fn fetch_props(&self) -> &Vec<GxlProp> {
        &self.props
    }
}

impl AppendAble<GxlProp> for GxlIntercept {
    fn append(&mut self, prop: GxlProp) {
        self.props.push(prop);
    }
}
