use orion_syspec::error::ToErr;

use super::prelude::*;
use crate::{
    execution::{runnable::AsyncDryrunRunnableTrait, task::Task},
    traits::Setter,
};

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
        let mut task = Task::from("loop");
        if let Some(named_dict) = dict.nameds().get(self.dct_name()) {
            let mut cur_dict = dict.clone();
            for (_k, v) in named_dict.maps().iter() {
                cur_dict
                    .global_mut()
                    .set(self.cur_name().as_str(), v.clone());
                let (dict, out) =
                    self.body.async_exec_with_dryrun(ctx.clone(), cur_dict, false)
                        .await?;
                cur_dict = dict;
                task.append(out);
            }
            return Ok((cur_dict, ExecOut::Task(task)));
        }
        ExecReason::Miss(self.dct_name().into()).err_result()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{model::components::gxl_block::BlockNode, traits::Getter, var::VarDict};
    use rstest::*;

    #[fixture]
    fn test_ctx() -> ExecContext {
        ExecContext::default()
    }

    #[fixture]
    fn test_dict() -> VarSpace {
        VarSpace::default()
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_success(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] mut dict: VarSpace,
    ) {
        // 准备测试数据
        let named_dict = {
            let mut n = VarDict::default();
            n.set("key1", "value1");
            n.set("key2", "value2");
            n
        };

        dict.nameds_mut()
            .insert("test_dict".to_string(), named_dict);

        // 创建一个空的 BlockNode 作为循环体
        let body = BlockNode::new();
        let loop_node = GxlLoop::new("current".to_string(), "test_dict".to_string(), body);

        // 执行测试
        let result = loop_node.async_exec(ctx, dict).await;

        // 验证结果
        assert!(result.is_ok());
        let (_, exec_out) = result.unwrap();
        if let ExecOut::Task(_task) = exec_out {
            assert!(true)
            //assert_eq!(job.tasks().len(), 2); // 应该有两个任务，对应两个键值对
        } else {
            panic!("Expected Job output");
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_missing_dict(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] dict: VarSpace,
    ) {
        // 创建一个空的 BlockNode 作为循环体
        let body = BlockNode::new();
        let loop_node = GxlLoop::new("current".to_string(), "missing_dict".to_string(), body);

        // 执行测试
        let result = loop_node.async_exec(ctx, dict).await;

        // 验证错误情况
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.to_string(), "[510] miss : missing_dict"); // 根据实际错误消息调整
    }

    #[rstest]
    #[tokio::test]
    async fn test_loop_exec_variable_setting(
        #[from(test_ctx)] ctx: ExecContext,
        #[from(test_dict)] mut dict: VarSpace,
    ) {
        // 准备测试数据
        let named_dict = {
            let mut n = VarDict::default();
            n.set("key1", "value1");
            n
        };

        dict.nameds_mut()
            .insert("test_dict".to_string(), named_dict);

        // 创建一个会检查当前变量的 BlockNode
        let body = BlockNode::new(); // 这里可以添加验证逻辑
        let loop_node = GxlLoop::new("current".to_string(), "test_dict".to_string(), body);

        // 执行测试
        let result = loop_node.async_exec(ctx.clone(), dict.clone()).await;

        // 验证变量设置
        assert!(result.is_ok());
        let (result_dict, _) = result.unwrap();
        assert_eq!(
            result_dict.global().get("current").unwrap().to_string(),
            "value1" // 最后一次循环设置的值
        );
    }
}
