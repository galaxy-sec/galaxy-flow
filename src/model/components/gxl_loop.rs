use orion_syspec::error::ToErr;

use super::prelude::*;
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
    async fn async_exec(&self, ctx: ExecContext, dict: VarSpace) -> VTResult {
        let mut job = Job::from("loop");
        if let Some(named_dict) = dict.nameds().get(self.dct_name()) {
            let mut cur_dict = dict.clone();
            for (_k, v) in named_dict.maps().iter() {
                cur_dict
                    .globle_mut()
                    .set(self.cur_name().as_str(), v.clone());
                let (dict, task) = self.body.async_exec(ctx.clone(), cur_dict).await?;
                cur_dict = dict;
                job.append(task);
            }
            return Ok((cur_dict, ExecOut::Job(job)));
        }
        ExecReason::Miss(self.dct_name().into()).err_result()
    }
}
