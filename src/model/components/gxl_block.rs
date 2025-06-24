use async_trait::async_trait;
use gag::BufferRedirect;

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
use crate::task_report::task_rc_config::TASK_REPORT_CENTER;

use super::gxl_cond::GxlCond;
use super::gxl_spc::GxlSpace;
use super::gxl_var::GxlProp;
use std::io;
use std::io::Read;

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

        // 创建输出重定向，如果任务报告中心启用则捕获标准输出
        let redirect: Option<io::Result<BufferRedirect>> = match TASK_REPORT_CENTER.get() {
            Some(task_config) if task_config.report_enable => {
                // 如果报告中心启用，则尝试创建重定向
                Some(BufferRedirect::stdout())
            }
            _ => {
                // 如果报告中心被禁用，则不创建重定向
                None
            }
        };

        // 对于GxlRun，我们需要在执行前取消重定向
        let (execution_result, captured_data) = match self {
            BlockAction::GxlRun(o) => {
                // 如果存在重定向，在执行GxlRun前读取并关闭它
                let output = if let Some(stdout_redirect) = redirect {
                    let mut output = String::new();
                    let mut stdout_capture = stdout_redirect
                        .map_err(|e| ExecError::from(ExecReason::Io(e.to_string())))?;
                    
                    stdout_capture.read_to_string(&mut output)
                        .map_err(|e| ExecError::from(ExecReason::Io(e.to_string())))?;
                    
                    // 确保在执行GxlRun前恢复标准输出
                    stdout_capture.into_inner();
                    output
                } else {
                    String::new()
                };
                
                // 执行GxlRun
                (o.async_exec(ctx, dct).await, output)
            },
            // 对于其他动作，保持重定向
            _ => {
                let action_res = match self {
                    BlockAction::Command(o) => o.async_exec_with_dryrun(ctx, dct, is_dryrun).await,
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
                    _ => unreachable!(),
                };

                // 处理重定向的输出
                let output = if let Some(stdout_redirect) = redirect {
                    let mut output = String::new();
                    let mut stdout_capture = stdout_redirect
                        .map_err(|e| ExecError::from(ExecReason::Io(e.to_string())))?;
                    
                    stdout_capture.read_to_string(&mut output)
                        .map_err(|e| ExecError::from(ExecReason::Io(e.to_string())))?;
                    
                    // 恢复标准输出
                    stdout_capture.into_inner();
                    output
                } else {
                    String::new()
                };

                (action_res, output)
            }
        };
        // 只在有实际输出时打印
        if !captured_data.trim().is_empty() {
            println!("{}", captured_data);
        }

        // 处理执行结果
        match execution_result {
            Ok((vars_dict, out)) => {
                let out = match out {
                    ExecOut::Action(mut act) => {
                        if !captured_data.is_empty() {
                            act.stdout.push_str(&captured_data);
                        }
                        ExecOut::Action(act)
                    }
                    ExecOut::Task(mut task) => {
                        if !captured_data.is_empty() {
                            // 如果task已有输出，则添加换行符
                            if !task.stdout.is_empty() {
                                task.stdout.push('\n');
                            }
                            task.stdout.push_str(&captured_data);
                        }
                        ExecOut::Task(task)
                    }
                    other => other,
                };
                Ok((vars_dict, out))
            }
            Err(e) => Err(e),
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