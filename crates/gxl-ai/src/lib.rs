//! GXL AI原生工作流引擎
//! GXL 2.0 - AI-Agent Workflow Language Core
//! 直接使用Rust原生与大模型交互，无需外部依赖

mod client;
mod models;
mod commands;
mod git_integration;
mod error;

use client::{AiClient, ModelProvider};
use error::{AiError, Result};
use models::{AiRequest, AiResponse};
use serde_json::json;

pub use commands::execute_ai_command;
pub use git_integration::{GitContext, ChangeAnalysis};

/// GXL AI原生指令系统的入口点
pub struct GxlAiEngine {
    client: AiClient,
    git_context: GitContext,
}

impl GxlAiEngine {
    /// 创建新的AI-Native工作流引擎实例
    pub fn new(openai_key: String, claude_key: Option<String>) -> Result<Self> {
        let client = AiClient::new(
            openai_key.clone(),
+            claude_key.clone(),
+        );
+        let git_context = GitContext::new()?;
+        Ok(Self {
+            client,
+            git_context,
+        })
+    }
+
+    /// GXL原生指令：ai.git_intent() 实现
+    pub async fn analyze_git_changes(&self, limit_context: usize) -> Result<AiResponse> {
+        let changes = self.git_context.get_staged_changes(limit_context).await?;
+
+        if changes.files.is_empty() {
+            return Err(AiError::NoChangesDetected);
+        }
+
+        let diff_text = changes.to_diff_string();
+        let context = self.git_context.generate_context_json();
+
+        let request = AiRequest {
+            model: ModelProvider::Gpt4o,
+            system_prompt: "你是一个资深工程师，基于代码变更生成50字中文Conventional Commit消息".to_string(),
+            user_prompt: format!("变更分析：\n{}\n\n项目上下文：\n{}", diff_text, context),
+            temperature: 0.7,
+            max_tokens: 50,
+        };
+
+        self.client.query(request).await
+    }
+
+    /// GXL原生指令：ai.code_quality() 实现
+    pub async fn analyze_code_quality(&self, files: Vec<String>) -> Result<AiResponse> {
+        let code_context = self.git_context.get_code_context(files).await?;
+        let request = AiRequest {
+            model: ModelProvider::Gpt4o,
+            system_prompt: "分析代码质量，专注安全、性能和可维护性，Markdown格式输出".to_string(),
+            user_prompt: format!("代码质量分析:\n{}", code_context),
+            temperature: 0.3,
+            max_tokens: 800,
+        };
+
+        self.client.query(request).await
+    }
+
+    /// GXL原生指令：ai.generate_changelog() 实现
+    pub async fn generate_changelog(&self, since_tag: Option<String>) -> Result<AiResponse> {
+        let changelog_data = self.git_context.get_changelog_data(since_tag).await?;
+
+        let request = AiRequest {
+            model: ModelProvider::Claude35,
+            system_prompt: "生成符合keepachangelog标准的变更日志，Markdown格式".to_string(),
+            user_prompt: format!("变更日志生成:\n{}", changelog_data),
+            temperature: 0.5,
+            max_tokens: 1000,
+        };
+
+        self.client.query(request).await
+    }
+
+    /// GXL原生指令：ai.prongenerate() 实现
+    pub async fn prepare_pull_request(&self) -> Result<AiResponse> {
+        let pr_context = self.git_context.get_pr_context().await?;
+
+        let request = AiRequest {
+            model: ModelProvider::Gpt4o,
+            system_prompt: "为GitHub PR生成清晰标题和详细描述，Markdown格式".to_string(),
+            user_prompt: format!("PR生成:\n{}", pr_context),
+            temperature: 0.6,
+            max_tokens: 600,
+        };
+
+        self.client.query(request).await
+    }
+
+    /// 智能模型路由（根据复杂度选择最优模型）
+    pub async fn smart_route(&self, prompt: String, complexity: u8) -> Result<AiResponse> {
+        let model = match complexity {
+            1..=3 => ModelProvider::Gpt4oMini,
+            4..=7 => ModelProvider::Gpt4o,
+            8..=10 => ModelProvider::Claude35,
+            _ => ModelProvider::Claude35,
+        };
+
+        let request = AiRequest {
+            model,
+            system_prompt:
