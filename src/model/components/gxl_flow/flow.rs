use crate::ability::prelude::TaskValue;
use crate::components::gxl_env::env::anns_from_option_dto;
use crate::components::gxl_spc::GxlSpace;
use crate::components::gxl_utls::mod_obj_name;
use crate::model::components::prelude::*;

use crate::annotation::{ComUsage, TaskMessage};
use crate::execution::runnable::{AsyncDryrunRunnableTrait, AsyncRunnableTrait};
use crate::execution::task::Task;
use crate::parser::stc_base::AnnDto;
use crate::task_report::task_notification::TaskNotice;
use crate::task_report::task_rc_config::{build_task_url, TaskUrlType};
use crate::task_report::task_result_report::TaskReport;
use crate::traits::DependTrait;

use crate::components::gxl_block::BlockNode;
use crate::util::http_handle::{create_and_send_task_notice, send_http_request};
use std::io::Write;
use std::sync::Arc;

use derive_getters::Getters;

use super::anno::FlowAnnFunc;
use super::meta::FlowMeta;
use super::runner::FlowRunner;

#[derive(Clone, Getters, Debug)]
pub struct GxlFlow {
    meta: FlowMeta,
    pre_flows: Vec<FlowRunner>,
    post_flows: Vec<FlowRunner>,
    blocks: Vec<BlockNode>,
}

impl DependTrait<&GxlSpace> for GxlFlow {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        let mut target = GxlFlow::from(self.meta().clone());
        let pre_order_flows = self.meta.preorder();
        let mut buffer = Vec::new();
        let mut linked = false;
        for flow_id in pre_order_flows {
            assemble_pipe(mod_name, flow_id, src, &mut target.pre_flows)?;
            let _ = write!(&mut buffer, "{} | ", flow_id);
            linked = true;
        }

        let _ = write!(&mut buffer, "{} ", self.meta().name());
        let post_order_flows = self.meta.postorder();
        for flow_id in post_order_flows {
            assemble_pipe(mod_name, flow_id, src, &mut target.post_flows)?;
            let _ = write!(&mut buffer, " | {} ", flow_id);
            linked = true;
        }
        if linked {
            info!(
                target: "assemble",
                "assemble flow {:>8}.{:<8}: {} ",
                mod_name,
                self.meta().name(),
                String::from_utf8(buffer).unwrap()
            );
        }
        //for prop in self.props() {
        //    target.append(prop.clone());
        //}
        for block in self.blocks {
            let full_block = block.assemble(mod_name, src)?;
            target.append(full_block);
        }
        Ok(target)
    }
}

fn assemble_pipe(
    m_name: &str,
    flow: &str,
    src: &GxlSpace,
    target: &mut Vec<FlowRunner>,
) -> AResult<()> {
    let (t_mod, flow_name) = mod_obj_name(m_name, flow);
    if let Some(flow) = src.get(&t_mod).and_then(|m| m.load_scope_flow(&flow_name)) {
        let linked_flow = flow.assemble(m_name, src)?;
        target.push(linked_flow);
        return Ok(());
    }
    Err(AssembleError::from(AssembleReason::Miss(format!(
        "{}.{}",
        m_name, flow
    ))))
}

impl From<FlowMeta> for GxlFlow {
    fn from(meta: FlowMeta) -> Self {
        Self {
            meta,
            pre_flows: Vec::new(),
            post_flows: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl From<&str> for GxlFlow {
    fn from(name: &str) -> Self {
        let meta = FlowMeta::build_flow(name);
        Self {
            meta,
            pre_flows: Vec::new(),
            post_flows: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl GxlFlow {
    pub fn set_anns(&mut self, dto: Option<AnnDto>) {
        self.meta.set_annotates(anns_from_option_dto(dto));
    }
    pub fn load_ins<S: Into<String>>(name: S) -> Self {
        Self {
            meta: FlowMeta::build_flow(name.into()),
            pre_flows: Vec::new(),
            post_flows: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl GxlFlow {
    async fn exec_self(&self, ctx: ExecContext, mut var_dict: VarSpace) -> VTResult {
        let task_description = self.task_description();
        let mut task = Task::from(self.meta.name());
        let mut task_notice = TaskNotice::new();
        if let Some(des) = task_description.clone() {
            task = Task::from(des);
            task_notice = create_and_send_task_notice(&task, &task_notice).await?;
        }

        // 执行块
        for item in &self.blocks {
            let TaskValue { vars, rec, .. } = item
                .async_exec_with_dryrun(ctx.clone(), var_dict, self.is_dryrun())
                .await?;
            var_dict = vars;
            task.append(rec);
        }
        task.finish();

        match task_description {
            Some(_) => {
                let url = build_task_url(TaskUrlType::TaskReport)
                    .await
                    .unwrap_or_default();
                let task_result = TaskReport::from_task_and_notice(task.clone(), task_notice);
                send_http_request(task_result.clone(), &url).await;
                Ok(TaskValue::from((var_dict, ExecOut::Task(task))))
            }
            None => Ok(TaskValue::from((var_dict, ExecOut::Ignore))),
        }
    }

    // 获取注解中的描述信息
    pub fn get_desan(&self) -> Option<String> {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.desp().is_some() {
                return ann.desp();
            }
        }
        None
    }

    // 获取注解中的描述信息
    pub fn task_description(&self) -> Option<String> {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.message().is_some() {
                return ann.message();
            }
        }
        None
    }

    pub fn is_dryrun(&self) -> bool {
        let annotation = self.meta.annotations();
        for ann in annotation {
            if ann.func == FlowAnnFunc::Dryrun {
                return true;
            }
        }
        false
    }
}
#[async_trait]
impl AsyncRunnableTrait for GxlFlow {
    async fn async_exec(&self, mut ctx: ExecContext, mut var_dict: VarSpace) -> VTResult {
        let des = self
            .task_description()
            .unwrap_or(self.meta.name().to_string());
        let mut job = Job::from(&des);
        ctx.append(self.meta.name());
        for pre in self.pre_flows() {
            let TaskValue { vars, rec, .. } = pre.async_exec(ctx.clone(), var_dict).await?;
            var_dict = vars;
            job.append(rec);
        }
        let TaskValue { vars, rec, .. } = self.exec_self(ctx.clone(), var_dict).await?;
        var_dict = vars;
        job.append(rec);
        for post in self.post_flows() {
            let TaskValue { vars, rec, .. } = post.async_exec(ctx.clone(), var_dict).await?;
            var_dict = vars;
            job.append(rec);
        }
        if self.task_description().is_none() {
            return Ok(TaskValue::from((var_dict, ExecOut::Ignore)));
        }
        Ok(TaskValue::from((var_dict, ExecOut::Job(job))))
    }
}
impl ComponentMeta for GxlFlow {
    fn com_meta(&self) -> GxlMeta {
        GxlMeta::Flow(self.meta.clone())
    }
}

impl AppendAble<BlockNode> for GxlFlow {
    fn append(&mut self, block: BlockNode) {
        self.blocks.push(block);
    }
}

pub type FlowHold = Arc<GxlFlow>;

#[cfg(test)]
mod tests {

    use orion_error::TestAssert;

    use crate::components::GxlMod;

    use super::*;

    #[test]
    fn test_assemble_com_without_dependencies() {
        // 创建一个 RgMod 实例
        let gxl_mod = GxlMod::from("test_mod");

        // 创建一个目标 RgFlow 实例，没有依赖关系
        let target_flow = GxlFlow::from("target_flow");

        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        // 调用 assemble_com 方法
        let assembled_flow = target_flow.assemble("test_mod", &spc).assert();

        // 验证 pre_ows 和 post_ows 是否为空
        assert_eq!(assembled_flow.pre_flows().len(), 0);
        assert_eq!(assembled_flow.post_flows().len(), 0);
    }

    #[test]
    fn test_assemble_com_with_missing_dependencies() {
        // 创建一个 RgMod 实例，包含部分依赖关系
        let mut gxl_mod = GxlMod::from("test_mod");

        // 创建一些 RgFlow 实例
        let flow1 = GxlFlow::from("flow1");
        let flow2 = GxlFlow::from("flow2");

        // 将这些 RgFlow 实例添加到 RgMod 中
        gxl_mod.append(flow1);
        gxl_mod.append(flow2);

        // 创建一个目标 RgFlow 实例，包含部分存在的依赖关系
        let mut target_flow = GxlFlow::from("target_flow");
        target_flow.meta.set_preorder(vec!["flow1", "non_existent"]);
        target_flow
            .meta
            .set_postorder(vec!["flow2", "non_existent2"]);

        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        // 调用 assemble_com 方法
        assert!(target_flow.assemble("test_mod", &spc).is_err());
    }

    #[test]
    fn test_assemble_com_with_multiple_dependencies() {
        // 创建一个 RgMod 实例，包含多个依赖关系
        let mut gxl_mod = GxlMod::from("test_mod");

        // 创建一些 RgFlow 实例
        let flow1 = GxlFlow::from(FlowMeta::build_flow("flow1".to_string()));
        let flow2 = GxlFlow::from(FlowMeta::build_flow("flow2".to_string()));
        let flow3 = GxlFlow::from(FlowMeta::build_flow("flow3".to_string()));

        // 将这些 RgFlow 实例添加到 RgMod 中
        gxl_mod.append(flow1);
        gxl_mod.append(flow2);
        gxl_mod.append(flow3);

        // 创建一个目标 RgFlow 实例，包含多个依赖关系
        let mut target_flow = GxlFlow::from("target_flow");
        target_flow.meta.set_preorder(vec!["flow1", "flow2"]);
        target_flow.meta.set_postorder(vec!["flow3"]);
        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);

        // 调用 assemble_com 方法
        let assembled_flow = target_flow.assemble("test_mod", &spc).assert();

        // 验证 pre_ows 和 post_ows 是否正确组装
        assert_eq!(assembled_flow.pre_flows().len(), 2);
        assert_eq!(assembled_flow.pre_flows()[0].flow().meta.name(), "flow1");
        assert_eq!(assembled_flow.pre_flows()[1].flow().meta.name(), "flow2");

        assert_eq!(assembled_flow.post_flows().len(), 1);
        assert_eq!(assembled_flow.post_flows()[0].flow().meta.name(), "flow3");
    }
}
