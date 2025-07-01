use crate::{
    ability::prelude::{GxlProp, TaskValue},
    annotation::{Dryrunable, Transaction},
    components::{gxl_intercept::GxlIntercept, gxl_spc::GxlSpace},
    execution::hold::TransableHold,
    model::components::prelude::*,
};

use super::flow::GxlFlow;

#[derive(Clone, Getters)]
pub struct FlowRunner {
    m_name: String,
    flow: GxlFlow,
    entry: GxlIntercept,
    exit: GxlIntercept,
}
//pub type FlowHold = Rc<FlowRunner>;
impl FlowRunner {
    pub(crate) fn new(
        m_name: String,
        props: Vec<GxlProp>,
        flow: GxlFlow,
        befores: Vec<GxlFlow>,
        afters: Vec<GxlFlow>,
    ) -> Self {
        Self {
            entry: GxlIntercept::new(m_name.clone(), props, befores),
            flow,
            exit: GxlIntercept::new(m_name.clone(), Vec::new(), afters),
            m_name,
        }
    }
}

impl DependTrait<&GxlSpace> for FlowRunner {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        Ok(Self {
            m_name: self.m_name,
            entry: self.entry.assemble(mod_name, src)?,
            flow: self.flow.assemble(mod_name, src)?,
            exit: self.exit.assemble(mod_name, src)?,
        })
    }
}

impl Transaction for FlowRunner {
    fn is_transaction(&self) -> bool {
        self.flow.is_transaction()
    }
    fn undo_hold(&self) -> Option<TransableHold> {
        self.flow.undo_hold()
    }
}

impl Dryrunable for FlowRunner {
    fn dryrun_hold(&self) -> Option<TransableHold> {
        self.flow.dryrun_hold()
    }
}

#[async_trait]
impl AsyncRunnableTrait for FlowRunner {
    async fn async_exec(&self, mut ctx: ExecContext, dict: VarSpace) -> TaskResult {
        //let orgion = dict.clone();
        let mut job = Job::from("scope_flow");
        ctx.append(self.m_name.as_str());
        // 使用链式调用和模式匹配
        let dict = {
            let TaskValue { vars, rec, .. } = self.entry().async_exec(ctx.clone(), dict).await?;
            job.append(rec);
            vars
        };

        let dict = {
            let TaskValue { vars, rec, .. } = self.flow().async_exec(ctx.clone(), dict).await?;
            job.append(rec);
            vars
        };

        let dict = {
            let TaskValue { vars, rec, .. } = self.exit().async_exec(ctx.clone(), dict).await?;
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
