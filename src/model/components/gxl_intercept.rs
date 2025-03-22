use super::gxl_spc::GxlSpace;
use super::{prelude::*, GxlFlow};

use super::gxl_var::RgProp;

#[derive(Clone, Getters, Debug)]
pub struct RgIntercept {
    m_name: String,
    props: Vec<RgProp>,
    flows: Vec<GxlFlow>,
}

impl RgIntercept {
    pub fn new(m_name: String, props: Vec<RgProp>, flows: Vec<GxlFlow>) -> Self {
        Self {
            m_name,
            props,
            flows,
        }
    }
}

impl RunnableTrait for RgIntercept {
    fn exec(&self, ctx: ExecContext, var_dict: &mut VarsDict) -> EOResult {
        self.export_props(ctx.clone(), var_dict, self.m_name())?;
        let mut job = Job::from("intercept");
        for flow in &self.flows {
            job.append(flow.exec(ctx.clone(), var_dict)?);
        }
        Ok(ExecOut::Job(job))
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
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

impl AppendAble<RgProp> for RgIntercept {
    fn append(&mut self, prop: RgProp) {
        self.props.push(prop);
    }
}

#[derive(Clone, Getters, Debug)]
pub struct RgFlowRunner {
    m_name: String,
    flow: GxlFlow,
    before: RgIntercept,
    after: RgIntercept,
}
impl RgFlowRunner {
    pub(crate) fn new(
        m_name: String,
        props: Vec<RgProp>,
        flow: GxlFlow,
        befores: Vec<GxlFlow>,
        afters: Vec<GxlFlow>,
    ) -> Self {
        Self {
            before: RgIntercept::new(m_name.clone(), props, befores),
            flow,
            after: RgIntercept::new(m_name.clone(), Vec::new(), afters),
            m_name,
        }
    }
}

impl DependTrait<&GxlSpace> for RgFlowRunner {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        Ok(Self {
            m_name: self.m_name,
            before: self.before.assemble(mod_name, src)?,
            flow: self.flow.assemble(mod_name, src)?,
            after: self.after.assemble(mod_name, src)?,
        })
    }
}

impl RunnableTrait for RgFlowRunner {
    fn exec(&self, mut ctx: ExecContext, dict: &mut VarsDict) -> EOResult {
        let mut job = Job::from("scope_flow");
        ctx.append(self.m_name.as_str());
        job.append(self.before().exec(ctx.clone(), dict)?);
        job.append(self.flow().exec(ctx.clone(), dict)?);
        job.append(self.after().exec(ctx.clone(), dict)?);
        Ok(ExecOut::Job(job))
    }
}
impl ComponentRunnable for RgFlowRunner {
    fn meta(&self) -> RgoMeta {
        self.flow().meta().clone()
    }
}
