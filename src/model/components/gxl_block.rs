use async_trait::async_trait;

use super::gxl_loop::GxlLoop;
use super::prelude::*;

use crate::ability::artifact::GxArtifact;
use crate::ability::delegate::ActCall;
use crate::ability::GxAssert;
use crate::ability::GxCmd;
use crate::ability::GxDownLoad;
use crate::ability::GxEcho;
use crate::ability::GxRead;
use crate::ability::GxRun;
use crate::ability::GxTpl;
use crate::ability::GxUpLoad;
use crate::ability::RgVersion;
use crate::calculate::cond::CondExec;
use crate::context::ExecContext;
use crate::execution::runnable::AsyncDryrunRunnableTrait;
use crate::execution::runnable::VTResult;
use crate::execution::task::Task;

use super::gxl_cond::GxlCond;
use super::gxl_spc::GxlSpace;
use super::gxl_var::GxlProp;

#[derive(Clone, Debug)]
pub enum BlockAction {
    Command(GxCmd),
    GxlRun(GxRun),
    Cond(GxlCond),
    Loop(GxlLoop),
    Echo(GxEcho),
    Assert(GxAssert),
    Version(RgVersion),
    Read(GxRead),
    Delegate(ActCall),
    Tpl(GxTpl),
    Artifact(GxArtifact),
    DownLoad(GxDownLoad),
    UpLoad(GxUpLoad),
}

#[derive(Clone, Getters, Default, Debug)]
pub struct BlockNode {
    props: Vec<GxlProp>,
    items: Vec<BlockAction>,
}

impl BlockNode {
    pub fn new() -> Self {
        Self {
            props: vec![],
            items: vec![],
        }
    }
}

#[async_trait]
impl CondExec for BlockNode {
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> VTResult {
        self.async_exec_with_dryrun(ctx, def, false).await
    }
}
#[async_trait]
impl AsyncDryrunRunnableTrait for BlockAction {
    async fn async_exec_with_dryrun(
        &self,
        ctx: ExecContext,
        dct: VarSpace,
        is_dryrun: bool,
    ) -> VTResult {
        match self {
            BlockAction::Command(o) => o.async_exec_with_dryrun(ctx, dct, is_dryrun).await,
            BlockAction::GxlRun(o) => o.async_exec(ctx, dct).await,
            BlockAction::Echo(o) => o.async_exec(ctx, dct).await,
            BlockAction::Assert(o) => o.async_exec(ctx, dct).await,
            BlockAction::Cond(o) => o.async_exec(ctx, dct).await,
            BlockAction::Loop(o) => o.async_exec(ctx, dct).await,
            BlockAction::Tpl(o) => o.async_exec(ctx, dct).await,
            BlockAction::Delegate(o) => o.async_exec(ctx, dct).await,
            BlockAction::Version(o) => o.async_exec(ctx, dct).await,
            BlockAction::Read(o) => o.async_exec(ctx, dct).await,
            BlockAction::Artifact(o) => o.async_exec(ctx, dct).await,
            BlockAction::UpLoad(o) => o.async_exec(ctx, dct).await,
            BlockAction::DownLoad(o) => o.async_exec(ctx, dct).await,
        }
    }
}

#[async_trait]
impl AsyncDryrunRunnableTrait for BlockNode {
    async fn async_exec_with_dryrun(
        &self,
        ctx: ExecContext,
        var_dict: VarSpace,
        is_dryrun: bool,
    ) -> VTResult {
        //ctx.append("block");
        let mut task = Task::from("block");
        let mut cur_var_dict = var_dict;
        self.export_props(ctx.clone(), cur_var_dict.global_mut(), "")?;

        for item in &self.items {
            let (tmp_var_dict, out) = item
                .async_exec_with_dryrun(ctx.clone(), cur_var_dict, is_dryrun)
                .await?;
            cur_var_dict = tmp_var_dict;
            task.append(out);
        }
        task.finish();
        Ok((cur_var_dict, ExecOut::Task(task)))
    }
}
impl DependTrait<&GxlSpace> for BlockNode {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        //let mut ins = BlockNode::default();
        //ins.props = self.props().clone();
        let mut ins = BlockNode {
            props: self.props().clone(),
            ..Default::default()
        };
        for x in self.items {
            let item = match x {
                BlockAction::Tpl(v) => BlockAction::Tpl(v.clone()),
                BlockAction::Cond(v) => BlockAction::Cond(v.clone()),
                BlockAction::Loop(v) => BlockAction::Loop(v.clone()),
                BlockAction::Read(v) => BlockAction::Read(v.clone()),
                BlockAction::Echo(v) => BlockAction::Echo(v.clone()),
                //BlockAction::Vault(v) => BlockAction::Vault(v.clone()),
                BlockAction::Assert(v) => BlockAction::Assert(v.clone()),
                BlockAction::Version(v) => BlockAction::Version(v.clone()),
                BlockAction::Command(v) => BlockAction::Command(v.clone()),
                BlockAction::GxlRun(v) => BlockAction::GxlRun(v.clone()),
                BlockAction::Delegate(v) => BlockAction::Delegate(v.assemble(mod_name, src)?),
                BlockAction::Artifact(v) => BlockAction::Artifact(v.clone()),
                BlockAction::DownLoad(v) => BlockAction::DownLoad(v.clone()),
                BlockAction::UpLoad(v) => BlockAction::UpLoad(v.clone()),
            };
            ins.append(item);
        }
        Ok(ins)
    }
}
impl PropsTrait for BlockNode {
    fn fetch_props(&self) -> &Vec<GxlProp> {
        &self.props
    }
}

impl AppendAble<GxlProp> for BlockNode {
    fn append(&mut self, prop: GxlProp) {
        self.props.push(prop);
    }
}
impl AppendAble<BlockAction> for BlockNode {
    fn append(&mut self, hold: BlockAction) {
        self.items.push(hold);
    }
}
impl AppendAble<Vec<BlockAction>> for BlockNode {
    fn append(&mut self, holds: Vec<BlockAction>) {
        for h in holds {
            self.items.push(h);
        }
    }
}

#[cfg(test)]
mod tests {

    use orion_common::friendly::New2;

    //test RgBlock append
    use super::*;
    use crate::model::components::gxl_block::BlockNode;
    #[test]
    fn test_append() {
        let mut block = BlockNode::new();
        let prop = GxlProp::new("test", "hello");
        block.append(prop);
        assert_eq!(block.props.len(), 1);
    }
    //test RgBlock exec method
    #[tokio::test]
    async fn test_exec() {
        let mut block = BlockNode::new();
        let prop = GxlProp::new("test", "hello");
        block.append(prop);
        let ctx = ExecContext::new(false, false);
        let def = VarSpace::default();
        let res = block.async_exec_with_dryrun(ctx, def, false).await;
        assert_eq!(res.is_ok(), true);
    }
}
