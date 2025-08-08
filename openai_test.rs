use galaxy_flow::ai::config::AiConfig;
use galaxy_flow::ai::{AiCapability, AiClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("🚀 GXL AI-Native Runtime Test");
    println!("=================================\n");

    // 检查 API key
    let api_key = std::env::var("OPENAI_API_KEY").ok();
    if api_key.is_none() {
        println!("⚠️  请先设置环境变量: export OPENAI_API_KEY=your_key");
        println!("   或使用本地 AI: OLLAMA_MODEL=deepseek-coder");

        // 创建配置（会尝试从环境加载）
        let config = AiConfig::load()?;
        let client = AiClient::new(config)?;

        // 使用 Mock 测试
        let response = client
            .smart_request(AiCapability::Commit, "添加了用户认证功能")
            .await?;

        println!("🔧 Mock Response: {}", response.content);
        return Ok(());
    }

    println!("✅ 检测到 OpenAI API 密钥");

    // 配置和初始化
    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    // 测试1: 智能代码理解
    println!("📊 测试1: 代码分析");
    let response = client
        .smart_request(
            AiCapability::Analyze,
            "fn fib(n: i32) -> i32 { if n <= 1 { n } else { fib(n-1) + fib(n-2) } }",
        )
        .await?;
    println!("🧠 分析结果:\n{}", response.content);

    // 测试2: 智能Git提交
    println!("\n📝 测试2: Git智能提交");
    let diff_context = "修改了错误处理逻辑，添加了用户输入验证";
    let commit = client.smart_commit(diff_context).await?;
    println!("🎯 生成的提交信息: {}", commit.content);

    // 测试3: 代码审查
    println!("\n🔍 测试3: 代码审查");
    let review = client
        .code_review("fn divide(a: i32, b: i32) -> i32 { a / b }", "math.rs")
        .await?;
    println!("📋 审查结果:\n{}", review.content);

    println!("\n✅ 所有测试通过！GXL AI-Native 验证成功");
    Ok(())
}
