use super::prelude::*;
use crate::calculate::dynval::{EnvVarTag, VarCalcSupport};
use crate::traits::Setter;

use super::gxl_block::BlockNode;

#[derive(Clone, Getters, Debug)]
pub struct GxlLoop {
    cur_name: String,
    dct_name: String,
    body: BlockNode,
}

impl GxlLoop {
    pub fn new(cur_name: String, dct_name: String, body: BlockNode) -> Self {
        Self {
            cur_name,
            dct_name,
            body,
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for GxlLoop {
    async fn async_exec(&self, ctx: ExecContext, dict: VarsDict) -> VTResult {
        EnvVarTag::clear_import(&dict.export());
        let mut job = Job::from("loop");
        let mut cur_dict = VarsDict::default();
        for (k, v) in dict.maps().iter() {
            cur_dict.set(self.cur_name().as_str(), v.clone());
            let (dict, task) = self.body.async_exec(ctx.clone(), cur_dict).await?;
            cur_dict = dict;
            job.append(task);
        }
        Ok((cur_dict, ExecOut::Job(job)))
    }
}
