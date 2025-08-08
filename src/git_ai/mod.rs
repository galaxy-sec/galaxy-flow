//! Git AI工作流模块
//! 提供基于AI的Git操作自动化

mod changelog;
mod code_review;
mod smart_commit;

pub use changelog::AutoChangelog;
pub use code_review::CodeReviewFlow;
pub use smart_commit::SmartCommitFlow;

use crate::ai::AiClient;

/// Git AI配置
#[derive(Debug, Clone)]
pub struct GitAiConfig {
    pub auto_mode: bool,
    pub interactive: bool,
    pub dry_run: bool,
}

impl Default for GitAiConfig {
    fn default() -> Self {
        Self {
            auto_mode: false,
            interactive: true,
            dry_run: false,
        }
    }
}

/// Git AI CLI命令执行器
pub struct GitAiCommands {
    client: AiClient,
    config: GitAiConfig,
}

impl GitAiCommands {
    pub fn new(client: AiClient) -> Self {
        Self {
            client,
            config: GitAiConfig::default(),
        }
    }

    /// 运行智能提交
    pub async fn run_smart_commit(&self) -> crate::ai::error::AiResult<()> {
        let flow = SmartCommitFlow::new(self.client.clone(), self.config.auto_mode);
        flow.execute().await
    }

    /// 运行代码审查
    pub async fn run_code_review(&self) -> crate::ai::error::AiResult<()> {
        let flow = CodeReviewFlow::new(self.client.clone());
        flow.execute().await
    }

    /// 自动生成CHANGELOG
    pub async fn run_auto_changelog(&self) -> crate::ai::error::AiResult<()> {
        let generator = AutoChangelog::new(self.client.clone());
        generator.generate().await
    }
}
