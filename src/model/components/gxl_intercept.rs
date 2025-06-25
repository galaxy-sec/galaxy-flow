use super::gxl_spc::GxlSpace;
use super::{prelude::*, GxlFlow};

use super::gxl_var::GxlProp;

#[derive(Clone, Getters, Debug)]
pub struct RgIntercept {
    m_name: String,
    props: Vec<GxlProp>,
    flows: Vec<GxlFlow>,
}

impl RgIntercept {
    pub fn new(m_name: String, props: Vec<GxlProp>, flows: Vec<GxlFlow>) -> Self {
        Self {
            m_name,
            props,
            flows,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for RgIntercept {
    async fn async_exec(&self, ctx: ExecContext, mut var_dict: VarSpace) -> VTResult {
        let mut job = Job::from("intercept");
        self.export_props(ctx.clone(), var_dict.global_mut(), self.m_name())?;
        for flow in &self.flows {
            let (cur_dict, task) = flow.async_exec(ctx.clone(), var_dict).await?;
            var_dict = cur_dict;
            job.append(task);
        }
        Ok((var_dict, ExecOut::Job(job)))
    }
}
impl DependTrait<&GxlSpace> for RgIntercept {
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

impl PropsTrait for RgIntercept {
    fn fetch_props(&self) -> &Vec<GxlProp> {
        &self.props
    }
}

impl AppendAble<GxlProp> for RgIntercept {
    fn append(&mut self, prop: GxlProp) {
        self.props.push(prop);
    }
}
