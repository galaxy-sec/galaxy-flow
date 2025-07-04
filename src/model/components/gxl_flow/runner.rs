use crate::{
    ability::prelude::{GxlProp, TaskValue},
    components::{gxl_intercept::RgIntercept, gxl_spc::GxlSpace},
    model::components::prelude::*,
};

use super::flow::GxlFlow;

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
        props: Vec<GxlProp>,
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
            let TaskValue { vars, rec, .. } = self.before().async_exec(ctx.clone(), dict).await?;
            job.append(rec);
            vars
        };

        let dict = {
            let TaskValue { vars, rec, .. } = self.flow().async_exec(ctx.clone(), dict).await?;
            job.append(rec);
            vars
        };

        let dict = {
            let TaskValue { vars, rec, .. } = self.after().async_exec(ctx.clone(), dict).await?;
            job.append(rec);
            vars
        };
        Ok(TaskValue::from((dict, ExecOut::Job(job))))
    }
}
impl ComponentMeta for FlowRunner {
    fn com_meta(&self) -> GxlMeta {
        self.flow().com_meta()
    }
}
