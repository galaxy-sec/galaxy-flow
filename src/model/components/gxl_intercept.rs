use crate::ability::prelude::ComponentMeta;

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

#[async_trait]
impl AsyncRunnableTrait for RgIntercept {
    async fn async_exec(&self, ctx: ExecContext, mut var_dict: VarSpace) -> VTResult {
        let mut job = Job::from("intercept");
        self.export_props(ctx.clone(), var_dict.globle_mut(), self.m_name())?;
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
pub struct FlowRunner {
    m_name: String,
    flow: GxlFlow,
    before: RgIntercept,
    after: RgIntercept,
}
impl FlowRunner {
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

impl DependTrait<&GxlSpace> for FlowRunner {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        Ok(Self {
            m_name: self.m_name,
            before: self.before.assemble(mod_name, src)?,
            flow: self.flow.assemble(mod_name, src)?,
            after: self.after.assemble(mod_name, src)?,
        })
    }
}

#[async_trait]
impl AsyncRunnableTrait for FlowRunner {
    async fn async_exec(&self, mut ctx: ExecContext, dict: VarSpace) -> VTResult {
        //let orgion = dict.clone();
        let mut job = Job::from("scope_flow");
        ctx.append(self.m_name.as_str());
        // 使用链式调用和模式匹配
        let dict = {
            let (d, t) = self.before().async_exec(ctx.clone(), dict).await?;
            job.append(t);
            d
        };

        let dict = {
            let (d, t) = self.flow().async_exec(ctx.clone(), dict).await?;
            job.append(t);
            d
        };

        let dict = {
            let (d, t) = self.after().async_exec(ctx.clone(), dict).await?;
            job.append(t);
            d
        };
        Ok((dict, ExecOut::Job(job)))
    }
}
impl ComponentMeta for FlowRunner {
    fn com_meta(&self) -> GxlMeta {
        self.flow().meta().clone()
    }
}
