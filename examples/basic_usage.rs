//! AiExecUnit 基本使用示例
//!
//! 这个示例展示了如何使用 AiExecUnit 来执行AI任务，
//! 包括创建执行单元、执行请求和处理结果。

use galaxy_flow::ability::ai::ai_executor::AiExecutor;
use orion_ai::{AiConfig, AiExecUnit, AiExecUnitBuilder, AiResult};

#[tokio::main]
async fn main() -> AiResult<()> {
    println!("=== AiExecUnit 基本使用示例 ===\n");

    // 示例 1: 使用示例配置创建 AiExecUnit
    println!("1. 使用示例配置创建 AiExecUnit");
    let exec_unit = create_example_exec_unit().await?;
    println!("✓ AiExecUnit 创建成功\n");

    // 示例 2: 执行简单的AI请求
    println!("2. 执行简单的AI请求");
    let prompt = "你好，请简单介绍一下你自己";
    let result = exec_unit.execute(prompt).await?;
    println!("✓ AI回复: {}\n", result.content);

    // 示例 3: 使用 AiExecutor (更高级的用法)
    println!("3. 使用 AiExecutor (封装了 AiExecUnit)");
    basic_ai_executor_example().await?;

    println!("=== 示例完成 ===");
    Ok(())
}

/// 创建一个示例 AiExecUnit
async fn create_example_exec_unit() -> AiResult<AiExecUnit> {
    // 使用示例配置
    let config = AiConfig::example();

    // 使用构建器创建执行单元
    let exec_unit = AiExecUnitBuilder::new()
        .with_config(config)
        .with_role("assistant") // 设置角色为助手
        .build()?;

    Ok(exec_unit)
}

/// 展示 AiExecutor 的基本用法
async fn basic_ai_executor_example() -> AiResult<()> {
    use galaxy_flow::ability::prelude::*;

    // 创建 AiExecutor
    let mut executor = AiExecutor::default();

    // 设置任务和角色
    executor.set_task(Some("请用一句话描述 Rust 语言的特点".to_string()));
    executor.set_role(Some("developer".to_string()));

    // 创建执行上下文
    let ctx = ExecContext::new(None, false);
    let vars = galaxy_flow::ability::prelude::VarSpace::sys_init()
        .map_err(|e| orion_ai::error_utils::config_error(format!("初始化变量空间失败: {}", e)))?;

    // 执行AI任务
    match executor.async_exec(ctx, vars).await {
        Ok(task_value) => {
            let vars = &task_value.vars;
            if let Some(ai_result) = vars.get("AI") {
                println!("✓ AiExecutor 执行成功");
                println!("  AI回复: {:?}", ai_result);
            } else {
                println!("⚠ 未找到AI结果");
            }
        }
        Err(e) => {
            println!("⚠ AiExecutor 执行失败: {}", e);
            println!("  这可能是因为缺少有效的API密钥");
        }
    }

    Ok(())
}

/// 展示带工具的 AiExecUnit 用法
