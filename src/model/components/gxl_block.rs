use async_trait::async_trait;

use super::gxl_loop::GxlLoop;
use super::prelude::*;

use crate::ability::artifact::GxArtifact;
use crate::ability::delegate::ActCall;
use crate::ability::read::ReadMode;
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
use crate::execution::runnable::VTResult;
use crate::execution::task::Task;
use crate::model::task_result::TaskResult;
use crate::util::http_handle::get_task_callback_center_url;
use crate::util::http_handle::send_http_request;

use super::gxl_cond::GxlCond;
use super::gxl_spc::GxlSpace;
use super::gxl_var::RgProp;

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
    props: Vec<RgProp>,
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
        self.async_exec(ctx, def).await
    }
}
#[async_trait]
impl AsyncRunnableTrait for BlockAction {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> VTResult {
        let mut task_name = String::new();
        let res: Result<(VarSpace, ExecOut), orion_error::StructError<ExecReason>> = match self {
            BlockAction::Command(o) => {
                task_name = String::from("gx.cmd");
                o.async_exec(ctx, dct).await
            }
            BlockAction::GxlRun(o) => {
                task_name = String::from("gx.run");
                o.async_exec(ctx, dct).await
            }
            BlockAction::Echo(o) => o.async_exec(ctx, dct).await,
            BlockAction::Assert(o) => o.async_exec(ctx, dct).await,
            BlockAction::Cond(o) => o.async_exec(ctx, dct).await,
            BlockAction::Loop(o) => o.async_exec(ctx, dct).await,
            BlockAction::Tpl(o) => {
                task_name = String::from("build tpl file");
                o.async_exec(ctx, dct).await
            }
            BlockAction::Delegate(o) => {
                task_name = o.name.clone();
                o.async_exec(ctx, dct).await
            }
            BlockAction::Version(o) => o.async_exec(ctx, dct).await,
            BlockAction::Read(o) => {
                task_name = match o.imp() {
                    ReadMode::CMD(_) => String::from("gx.read_cmd"),
                    ReadMode::FILE(_) => String::from("gx.read_file"),
                    ReadMode::STDIN(_) => String::from("gx.read_ini"),
                    _ => String::from("unkown"),
                };
                o.async_exec(ctx, dct).await
            }
            BlockAction::Artifact(o) => o.async_exec(ctx, dct).await,
            BlockAction::UpLoad(o) => {
                task_name = String::from("gx.upload");
                o.async_exec(ctx, dct).await
            }
            BlockAction::DownLoad(o) => {
                task_name = String::from("gx.download");
                o.async_exec(ctx, dct).await
            }
        };
        if !task_name.is_empty() {
            // 若环境变量或配置文件中有返回路径则进行返回
            if let Some(url) = get_task_callback_center_url() {
                let task_result = TaskResult::from_result(task_name, &res);
                send_http_request(task_result.clone(), &url).await?;
                println!("task_result:{:#?}", task_result);
            }
        }
        res
    }
}

#[async_trait]
impl AsyncRunnableTrait for BlockNode {
    async fn async_exec(&self, ctx: ExecContext, var_dict: VarSpace) -> VTResult {
        //ctx.append("block");
        let mut task = Task::from("block");
        let mut cur_var_dict = var_dict;
        self.export_props(ctx.clone(), cur_var_dict.global_mut(), "")?;

        for item in &self.items {
            let (tmp_var_dict, out) = item.async_exec(ctx.clone(), cur_var_dict).await?;
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
    fn fetch_props(&self) -> &Vec<RgProp> {
        &self.props
    }
}

impl AppendAble<RgProp> for BlockNode {
    fn append(&mut self, prop: RgProp) {
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
        let prop = RgProp::new("test", "hello");
        block.append(prop);
        assert_eq!(block.props.len(), 1);
    }
    //test RgBlock exec method
    #[tokio::test]
    async fn test_exec() {
        let mut block = BlockNode::new();
        let prop = RgProp::new("test", "hello");
        block.append(prop);
        let ctx = ExecContext::new(false);
        let def = VarSpace::default();
        let res = block.async_exec(ctx, def).await;
        assert_eq!(res.is_ok(), true);
    }
}
