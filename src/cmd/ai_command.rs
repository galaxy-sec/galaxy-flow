use clap::{Parser, Subcommand};
use std::env;
use tokio;

use galaxy_flow::ai::{AiClient, AiConfig, AiCapability};

#[derive(Parser)]
#[command(name = "gx-ai", version = "1.0", about = "GXL AI-Native 工作流引擎")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// 智能Git提交 - 理解代码变更并生成提交信息
    SmartCommit {
        /// 自动模式，无需确认
        #[arg(long)]
        auto: bool,

        /// 干跑模式，仅显示建议信息，不实际提交
        #[arg(long)]
        dry_run: bool,
    },

    /// 代码审审查 - 智能分析变更代码质量
    CodeReview {
        /// 指定目标文件
        #[arg(short, long)]
        files: Vec<String>,
    },

    /// AI能力测试 - 快速测试AI连接
    Test {
        /// 指定测试消息
        #[arg(short, long, default_value = "Hello from GXL")]
        message: String,

        /// 指定使用的模型
        #[arg(short, long)]
        model: Option<String>,
    },

    /// 列出可用AI提供商和模型
    List,

    /// 生成项目CHANGELOG
    Changelog,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 检查基本环境
    if env::var("OPENAI_API_KEY").is_err() && env::var("CLAUDE_API_KEY").is_err() {
        println!("❌ 需要设置至少一个AI密钥:");
        println!("  OPENAI_API_KEY  或者  CLAUDE_API_KEY");
        println!("🎯 示例: export OPENAI_API_KEY='your-key-here'");
        return Ok(());
    }

    println!("🚀 GXL AI-Native 引擎启动...");

    // 初始化AI客户端
    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    let cli = Cli::parse();

    match cli.command {
        Commands::SmartCommit { auto, dry_run } => {
            smart_commit(auto, dry_run).await?;
        }
        Commands::CodeReview { files } => {
            code_review(files).await?;
        }
        Commands::Test { message, model } => {
            test_ai(client, message, model).await?;
        }
        Commands::List => {
            list_models(client).await?;
        }
        Commands::Changelog => {
            generate_changelog().await?;
        }
    }

    Ok(())
}

async fn smart_commit(auto: bool, dry_run: bool) -> Result<(), Box<dyn std::error::Error>> {
    use galaxy_flow::git_ai::SmartCommitFlow;

    println!("🎯 启动智能Git提交...");

    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    let flow = SmartCommitFlow::new(client, auto);

    if dry_run {
        println!("🏃 [{DRY RUN MODE}] - 仅显示建议，不执行提交");
    }

    flow.execute().await?;
    Ok(())
}

async fn code_review(files: Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    let config = AiConfig::load()?;
    let client = AiClient::new(config)?;

    println!("🔍 开始代码审查...");

    // 简单的代码审查示例
    for file in files.iter().take(3) { // 最多3个文件
        if let Ok(content) = std::fs::read_to_string(file) {
            let response = client
                .smart_request(AiCapability::Review, &format!("审查文件 {} 的代码:\n{}", file, content))
                .await?;

            println!("📄 文件 {} 的审查结果:", file);
            println!("{}", response.content);
            println!("-------------------");
        } else {
            println!("⚠️ 无法读取文件: {}", file);
        }
    }

    if files.is_empty() {
        println!("请指定要审查的文件，例如: gx-ai code-review --files main.rs utils.rs");
    }

    Ok(())
}

async fn test_ai(mut client: AiClient, message: String, model: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 AI连接测试...");

    let test_prompt = format!("简要回答：{}", message);

    let response = if let Some(model_name) = model {
        let request = galaxy_flow::ai::AiRequest::builder()
            .model(model_name)
            .user_prompt(test_prompt)
            .build();
        client.send_request(request).await?
    } else {
        client.smart_request(AiCapability::Suggest, &test_prompt).await?
    };

    println!("🎯 响应内容:");
    println!("{}", response.content);
    println!("💡 使用模型: {}", response.model);
    if let Some(cost) = response.usage.estimated_cost {
        println!("💰 成本: ${:.4}", cost);
    }

    Ok(())
}

async fn list_models(client: AiClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 可用AI提供商和模型:");

    for provider in client.available_providers() {
        println!("📊 {}:", provider);
    }

    println!("\n🛠️ 当前配置:");
    println!("  支持: OpenAI, Anthropic, Ollama");
    println!("  使用:设置OPENAI_API_KEY或CLAUDE_API_KEY");
    Ok(())
}

async fn generate_changelog() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔖 Changelog生成功能开发中...");
    println!("敬请期待智能更新日志生成！");
    Ok(())
}
+            .build();
+        client.send_request(request).
