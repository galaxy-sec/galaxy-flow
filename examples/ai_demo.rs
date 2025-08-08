use std::env;
use tokio;

use galaxy_flow::ai::{AiClient, AiConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 检查密钥
    if env::var("OPENAI_API_KEY").is_err() {
        println!("❌ 请设置环境变量 OPENAI_API_KEY");
        println!("示例: export OPENAI_API_KEY='你的密钥'");
        return Ok(());
    }

    println!("🚀 GXL AI 原生工作流演示启动...");

    // 加载配置
    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    println!("✅ AI客户端已启动");
    println!("🎯 可用provider: {:?}", client.available_providers());

    // 测试AI能力
    let test_prompt = "Explain what the future of AI-assisted coding looks like in one sentence.";

    match client
        .smart_request(
            galaxy_flow::ai::capabilities::AiCapability::Generate,
            test_prompt,
        )
        .await
    {
        Ok(response) => {
            println!("🎉 AI响应:");
            println!("{}", response.content);
            println!(
                "\n模型: {}, tokens: {}",
                response.model, response.usage.total_tokens
            );
            if let Some(cost) = response.usage.estimated_cost {
                println!("💰 预估成本: ${:.4}", cost);
            }
        }
        Err(e) => {
            println!("❌ 请求失败: {}", e);
        }
    }

    Ok(())
}
