use crate::{
    ability::prelude::RgProp,
    annotation::Transaction,
    components::{gxl_intercept::GxlIntercept, gxl_spc::GxlSpace},
    execution::hold::TransableHold,
    model::components::prelude::*,
};

use super::flow::GxlFlow;

#[derive(Clone, Getters)]
pub struct FlowRunner {
    m_name: String,
    flow: GxlFlow,
    before: GxlIntercept,
    after: GxlIntercept,
}
//pub type FlowHold = Rc<FlowRunner>;
impl FlowRunner {
    pub(crate) fn new(
        m_name: String,
        props: Vec<RgProp>,
        flow: GxlFlow,
        befores: Vec<GxlFlow>,
        afters: Vec<GxlFlow>,
    ) -> Self {
        Self {
            before: GxlIntercept::new(m_name.clone(), props, befores),
            flow,
            after: GxlIntercept::new(m_name.clone(), Vec::new(), afters),
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

impl Transaction for FlowRunner {
    fn is_transaction(&self) -> bool {
        self.flow.is_transaction()
    }
    fn undo_flow(&self) -> Option<TransableHold> {
        self.flow.undo_flow()
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
        self.flow().com_meta()
    }
}
