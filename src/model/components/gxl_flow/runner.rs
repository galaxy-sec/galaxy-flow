use crate::{
    ability::prelude::TaskValue,
    annotation::{Dryrunable, Transaction},
    components::gxl_spc::GxlSpace,
    execution::hold::TransableHold,
    model::components::prelude::*,
};

use super::flow::GxlFlow;

#[derive(Clone, Getters)]
pub struct FlowRunner {
    m_name: String,
    flow: GxlFlow,
}
//pub type FlowHold = Rc<FlowRunner>;
impl FlowRunner {
    pub(crate) fn new(m_name: String, flow: GxlFlow) -> Self {
        Self { flow, m_name }
    }
}

impl DependTrait<&GxlSpace> for FlowRunner {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        Ok(Self {
            m_name: self.m_name,
            flow: self.flow.assemble(mod_name, src)?,
        })
    }
}

impl Transaction for FlowRunner {
    fn is_transaction(&self) -> bool {
        self.flow.is_transaction()
    }
    fn undo_hold(&self) -> Vec<TransableHold> {
        self.flow.undo_hold()
    }
}

impl Dryrunable for FlowRunner {
    fn dryrun_hold(&self) -> Vec<TransableHold> {
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
            let TaskValue { vars, rec, .. } = self.flow().async_exec(ctx.clone(), dict).await?;
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
