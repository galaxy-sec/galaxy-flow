use crate::ability::prelude::{Action, TaskValue};
use crate::components::gxl_mod::meta::ModMeta;
use crate::components::gxl_spc::GxlSpace;
use crate::components::gxl_utls::mod_obj_name;
use crate::model::components::prelude::*;

use crate::execution::runnable::AsyncRunnableWithSenderTrait;
use crate::execution::task::Task;
use crate::task_report::task_notification::TaskNotice;
use crate::task_report::task_rc_config::{build_task_url, report_enable, TaskUrlType};
use crate::task_report::task_result_report::TaskReport;
use crate::traits::DependTrait;

use crate::components::gxl_block::BlockNode;
use crate::util::http_handle::send_http_request;
use crate::util::redirect::{init_redirect_file, read_log_content, seek_log_file_end, ReadSignal};
use contracts::requires;
use derive_getters::Getters;
use std::sync::{mpsc, Arc, Mutex};

use super::meta::FunMeta;

#[derive(Clone, Getters, Default)]
pub struct GxlFun {
    meta: FunMeta,
    blocks: Vec<BlockNode>,
    assembled: bool,
}
impl GxlFun {
    pub fn meta_mut(&mut self) -> &mut FunMeta {
        &mut self.meta
    }

    pub(crate) fn bind(&mut self, mod_meta: ModMeta) {
        self.meta.set_host(mod_meta);
    }
    pub fn with_mod(mut self, meta: ModMeta) -> Self {
        self.meta.set_host(meta);
        self
    }
    pub fn with_code(mut self, block: BlockNode) -> Self {
        self.blocks.push(block);
        self
    }
}

impl DependTrait<&GxlSpace> for GxlFun {
    fn assemble(self, mod_name: &str, src: &GxlSpace) -> AResult<Self> {
        debug!(target : "assemble", "will assemble flow {}" , self.meta().name() );
        let mut target = GxlFun::from(self.meta().clone());
        let mut buffer = Vec::new();
        let mut linked = false;

        if linked {
            info!(
                target: "assemble",
                "assemble flow  {} ",
                String::from_utf8(buffer).unwrap()
            );
        }
        for block in self.blocks {
            let full_block = block.assemble(mod_name, src)?;
            target.append(full_block);
        }
        target.assembled = true;
        debug!(target : "assemble", "assemble flow {} end" , target.meta().name() );
        Ok(target)
    }
}

fn assemble_fun_meta(m_name: &str, flow: &str, src: &GxlSpace) -> AResult<FunMeta> {
    let (mod_name, fun_name) = mod_obj_name(m_name, flow);
    debug!(target:"assemble", " find flow by {mod_name}.{fun_name}" );
    if let Some(flow) = src.get(&mod_name).and_then(|m| m.load_fun(&fun_name)) {
        debug!(target:"assemble", "found flow by {mod_name}.{fun_name}" );
        return Ok(flow.meta.clone());
    }
    Err(AssembleError::from(AssembleReason::Miss(format!(
        "{mod_name}.{fun_name}"
    ))))
}

impl From<FunMeta> for GxlFun {
    fn from(meta: FunMeta) -> Self {
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl From<&str> for GxlFun {
    fn from(name: &str) -> Self {
        let meta = FunMeta::build_fun(name);
        Self {
            meta,
            ..Default::default()
        }
    }
}

impl GxlFun {
    pub fn load_ins<S: Into<String>>(name: S) -> Self {
        Self {
            meta: FunMeta::build_fun(name.into()),
            ..Default::default()
        }
    }
}

impl GxlFun {
    #[requires(self.assembled )]
    async fn exec_self(
        &self,
        ctx: ExecContext,
        var_dict: VarSpace,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> TaskResult {
        let mut task = Task::from(self.meta.name());
        let mut task_notice = TaskNotice::new();
        // 执行所有块
        let (var_dict, task) = match self
            .execute_blocks(ctx, var_dict, None, task.clone(), task_notice.clone())
            .await
        {
            Ok((var_dict, task)) => (var_dict, task),
            Err(e) => {
                task.stdout.push_str(&e.to_string());
                task.result = Err(e.to_string());
                self.report_task_status(&task, &task_notice).await?;
                return Err(e);
            }
        };

        Ok(TaskValue::from((var_dict, ExecOut::Ignore)))
    }

    // 辅助方法：执行所有块
    async fn execute_blocks(
        &self,
        ctx: ExecContext,
        mut var_dict: VarSpace,
        task_description: Option<String>,
        mut task: Task,
        task_notice: TaskNotice,
    ) -> Result<(VarSpace, Task), ExecError> {
        for item in &self.blocks {
            if task_description.is_some() && report_enable().await {
                let (var_dict_new, task_new) = self
                    .execute_block_with_monitoring(
                        item,
                        ctx.clone(),
                        var_dict,
                        task_description.clone(),
                        task,
                        task_notice.clone(),
                    )
                    .await?;
                var_dict = var_dict_new;
                task = task_new;
            } else {
                let TaskValue { vars, rec, .. } =
                    item.async_exec(ctx.clone(), var_dict, None).await?;
                var_dict = vars;
                task.append(rec);
            }
        }
        task.finish();
        Ok((var_dict, task))
    }

    // 辅助方法：执行单个块并监控日志
    async fn execute_block_with_monitoring(
        &self,
        block: &BlockNode,
        ctx: ExecContext,
        var_dict: VarSpace,
        task_description: Option<String>,
        mut task: Task,
        task_notice: TaskNotice,
    ) -> Result<(VarSpace, Task), ExecError> {
        let (cur_sender, receiver) = mpsc::channel::<ReadSignal>();
        let log_file = init_redirect_file()?;
        let start_pos = Arc::new(Mutex::new(seek_log_file_end(&log_file)?));

        let shared_output = Arc::new(Mutex::new(String::new()));
        let shared_output_clone = Arc::clone(&shared_output);
        let mut shared_task = Arc::new(task.clone());
        let share_task_notice = Arc::new(task_notice.clone());
        let start_pos_clone = Arc::clone(&start_pos);
        let monitor_handle: tokio::task::JoinHandle<Result<(), ExecReason>> =
            tokio::spawn(async move {
                while let Ok(flag) = receiver.recv() {
                    match flag {
                        ReadSignal::Start(end) => {
                            let start = {
                                let guard = start_pos_clone
                                    .lock()
                                    .map_err(|e| ExecReason::Io(e.to_string()))?;
                                *guard
                            };
                            let buf = read_log_content(&log_file, start, end).await?;
                            if let Some(task_ref) = Arc::get_mut(&mut shared_task) {
                                task_ref.stdout.push_str(&buf);
                            }

                            let url = build_task_url(TaskUrlType::TaskReport)
                                .await
                                .unwrap_or_default();
                            let task_result = {
                                TaskReport::from_task_and_notice(
                                    (*shared_task).clone(),
                                    (*share_task_notice).clone(),
                                )
                            };
                            if let Ok(mut data) = shared_output_clone.lock() {
                                data.push_str(&buf);
                            }
                            send_http_request(task_result.clone(), &url).await;
                        }
                        ReadSignal::End(cur_start) => {
                            let mut guard = start_pos_clone
                                .lock()
                                .map_err(|e| ExecReason::Io(e.to_string()))?;
                            *guard = cur_start;
                        }
                    }
                }
                Ok(())
            });
        let sender_option = task_description.as_ref().map(|_| cur_sender.clone());
        let TaskValue { vars, rec, .. } = block.async_exec(ctx, var_dict, sender_option).await?;

        drop(cur_sender);
        drop(monitor_handle);

        task.append(rec);
        Self::update_task_with_output(&mut task, &shared_output, start_pos).await?;

        if task_description.is_some() {
            self.report_task_status(&task, &task_notice).await?;
        }

        Ok((vars, task))
    }

    /// 报告任务状态
    async fn report_task_status(
        &self,
        task: &Task,
        task_notice: &TaskNotice,
    ) -> Result<(), ExecReason> {
        let url = build_task_url(TaskUrlType::TaskReport)
            .await
            .unwrap_or_default();
        let task_result = TaskReport::from_task_and_notice(task.clone(), task_notice.clone());
        send_http_request(task_result, &url).await;
        Ok(())
    }

    /// 完成日志收集
    async fn finalize_log_collection(
        &self,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> Result<(), ExecReason> {
        if let Some(send) = sender {
            let log_file = init_redirect_file()?;
            let end_pos = seek_log_file_end(&log_file)?;
            send.send(ReadSignal::End(end_pos))
                .map_err(|e| ExecReason::Io(format!("flow send task error: {e}")))?;
        }
        Ok(())
    }

    // 更新任务输出
    async fn update_task_with_output(
        task: &mut Task,
        shared_output: &Arc<Mutex<String>>,
        start_pos: Arc<Mutex<u64>>,
    ) -> Result<(), ExecReason> {
        if let Ok(output) = shared_output.lock() {
            if !output.is_empty() {
                task.stdout = output.clone();
            }
        }

        let log_path = init_redirect_file()?;
        let end_pos = seek_log_file_end(&log_path)?;
        let start = *start_pos
            .lock()
            .map_err(|e| ExecReason::Io(e.to_string()))?;
        let content = read_log_content(&log_path, start, end_pos).await?;
        task.stdout.push_str(&content);

        Ok(())
    }
}
#[async_trait]
impl AsyncRunnableWithSenderTrait for GxlFun {
    async fn async_exec(
        &self,
        mut ctx: ExecContext,
        mut vars_dict: VarSpace,
        sender: Option<mpsc::Sender<ReadSignal>>,
    ) -> TaskResult {
        let mut action = Action::from("gx.cmd");
        ctx.append(self.meta.name());
        let TaskValue { vars, rec, .. } = self.exec_self(ctx.clone(), vars_dict, sender).await?;
        vars_dict = vars;
        Ok(TaskValue::from((vars_dict, ExecOut::Action(action))))
    }
}
impl ComponentMeta for GxlFun {
    fn gxl_meta(&self) -> GxlMeta {
        GxlMeta::Fun(self.meta.clone())
    }
}

impl AppendAble<BlockNode> for GxlFun {
    fn append(&mut self, block: BlockNode) {
        self.blocks.push(block);
    }
}

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
        let target_fun = GxlFun::from("target_fun");

        let mut spc = GxlSpace::default();
        spc.append(gxl_mod);
        // 调用 assemble_com 方法
        let assembled_fun = target_fun.assemble("test_mod", &spc).assert();

        // 验证 pre_ows 和 post_ows 是否为空
        assert_eq!(assembled_fun.meta().name(), "target_fun");
    }
}
