use async_trait::async_trait;
use gag::BufferRedirect;
use orion_error::ErrorOwe;

use super::gxl_loop::GxlLoop;
use super::prelude::*;

use crate::ability::artifact::GxArtifact;
use crate::ability::delegate::ActCall;
use crate::ability::prelude::TaskValue;
use crate::ability::shell::GxShell;
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
use crate::execution::runnable::TaskResult;
use crate::execution::task::Task;
use crate::task_report::task_rc_config::report_enable;

use super::gxl_cond::GxlCond;
use super::gxl_spc::GxlSpace;
use super::gxl_var::GxlVar;
use std::io::Read;

#[derive(Clone)]
pub enum BlockAction {
    Shell(GxShell),
    Command(GxCmd),
    GxlRun(GxRun),
    Cond(GxlCond),
    Loop(GxlLoop),
    Echo(GxEcho),
    Assert(GxAssert),
    Version(RgVersion),
    Read(GxRead),
    Delegate(Box<ActCall>),
    Tpl(GxTpl),
    Artifact(GxArtifact),
    DownLoad(GxDownLoad),
    UpLoad(GxUpLoad),
}

#[derive(Clone, Getters, Default)]
pub struct BlockNode {
    props: Vec<GxlVar>,
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
    async fn cond_exec(&self, ctx: ExecContext, def: VarSpace) -> TaskResult {
        self.async_exec(ctx, def).await
    }
}
#[async_trait]
impl AsyncRunnableTrait for BlockAction {
    async fn async_exec(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        let (result, output) = match self {
            BlockAction::GxlRun(o) => (o.async_exec(ctx, dct).await, String::new()),
            BlockAction::Loop(o) => (o.async_exec(ctx, dct).await, String::new()),
            _ => {
                let need_report = report_enable().await;
                if need_report {
                    let mut redirect = BufferRedirect::stdout().owe_sys()?;
                    let action_res = self.execute_action(ctx, dct).await;

                    let mut captured_output = String::new();
                    let _ = redirect.read_to_string(&mut captured_output);
                    redirect.into_inner();
                    (action_res, captured_output)
                } else {
                    let action_res = self.execute_action(ctx, dct).await;
                    (action_res, String::new())
                }
            } // 处理重定向的输出
        };

        print!("{}", output);
        return match result {
            Ok(mut tv) => {
                tv.append_out(output);
                Ok(tv)
            }
            Err(e) => Err(e),
        };
    }
}

impl BlockAction {
    /// 执行具体动作
    async fn execute_action(&self, ctx: ExecContext, dct: VarSpace) -> TaskResult {
        match self {
            BlockAction::Shell(o) => o.async_exec(ctx, dct).await,
            BlockAction::Command(o) => o.async_exec(ctx, dct).await,
            BlockAction::Echo(o) => o.async_exec(ctx, dct).await,
            BlockAction::Assert(o) => o.async_exec(ctx, dct).await,
            BlockAction::Cond(o) => o.async_exec(ctx, dct).await,
            BlockAction::Tpl(o) => o.async_exec(ctx, dct).await,
            BlockAction::Delegate(o) => o.async_exec(ctx, dct).await,
            BlockAction::Version(o) => o.async_exec(ctx, dct).await,
            BlockAction::Read(o) => o.async_exec(ctx, dct).await,
            BlockAction::Artifact(o) => o.async_exec(ctx, dct).await,
            BlockAction::UpLoad(o) => o.async_exec(ctx, dct).await,
            BlockAction::DownLoad(o) => o.async_exec(ctx, dct).await,
            _ => unreachable!(),
        }
    }
}

#[async_trait]
impl AsyncRunnableTrait for BlockNode {
    async fn async_exec(&self, ctx: ExecContext, var_dict: VarSpace) -> TaskResult {
        //ctx.append("block");
        let mut task = Task::from("block");
        let mut cur_var_dict = var_dict;
        self.export_props(ctx.clone(), cur_var_dict.global_mut(), "")?;

        for item in &self.items {
            let TaskValue { vars, out, rec } = item.async_exec(ctx.clone(), cur_var_dict).await?;
            cur_var_dict = vars;
            task.stdout.push_str(out.as_str());
            task.append(rec);
        }
        task.finish();
        Ok(TaskValue::from((cur_var_dict, ExecOut::Task(task))))
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
                BlockAction::Shell(v) => BlockAction::Shell(v.clone()),
                BlockAction::GxlRun(v) => BlockAction::GxlRun(v.clone()),
                BlockAction::Delegate(v) => {
                    BlockAction::Delegate(Box::new(v.assemble(mod_name, src)?))
                }
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
    fn fetch_props(&self) -> Vec<GxlVar> {
        self.props.clone()
    }
}

impl AppendAble<GxlVar> for BlockNode {
    fn append(&mut self, prop: GxlVar) {
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
        let prop = GxlVar::new("test", "hello");
        block.append(prop);
        assert_eq!(block.props.len(), 1);
    }
    //test RgBlock exec method
    #[tokio::test]
    async fn test_exec() {
        let mut block = BlockNode::new();
        let prop = GxlVar::new("test", "hello");
        block.append(prop);
        let ctx = ExecContext::new(Some(false), false);
        let def = VarSpace::default();
        let res = block.async_exec(ctx, def).await;
        assert!(res.is_ok());
    }
}
