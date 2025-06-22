use super::gxl_spc::GxlSpace;
use super::{prelude::*, GxlFlow};

use super::gxl_var::RgProp;

#[derive(Clone, Getters)]
pub struct GxlIntercept {
    m_name: String,
    props: Vec<RgProp>,
    flows: Vec<GxlFlow>,
}

impl GxlIntercept {
    pub fn new(m_name: String, props: Vec<RgProp>, flows: Vec<GxlFlow>) -> Self {
        Self {
            m_name,
            props,
            flows,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlIntercept {
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
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

impl AppendAble<RgProp> for GxlIntercept {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}
