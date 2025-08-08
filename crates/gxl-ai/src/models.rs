use serde::{Deserialize, Serialize};
use std::fmt;

/// GXL原生AI模型定义
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ModelProvider {
    Gpt4o,
    Gpt4oMini,
    Claude35,
    ClaudeHaiku,
    Ollama(&'static str),
}

impl fmt::Display for ModelProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModelProvider::Gpt4o => write!(f, "gpt-4o"),
            ModelProvider::Gpt4oMini => write!(f, "gpt-4o-mini"),
            ModelProvider::Claude35 => write!(f, "claude-3-5-sonnet-20241022"),
            ModelProvider::ClaudeHaiku => write!(f, "claude-3-haiku-20240307"),
            ModelProvider::Ollama(name) => write!(f, "{}", name),
        }
    }
}

/// GXL AI原生请求数据结构
#[derive(Debug, Clone, Serialize)]
pub struct AiRequest {
    pub model: ModelProvider,
    pub system_prompt: String,
    pub user_prompt: String,
    pub temperature: f32,
    pub max_tokens: u32,
}

impl AiRequest {
    /// 为Git Commit优化的便捷构造器
    pub fn for_git_commit(diff: &str, context: &str) -> Self {
        Self {
            model: ModelProvider::Gpt4o,
            system_prompt: "你是一名资深工程师，基于代码变更生成50字内中文Conventional Commit消息".to_string(),
            user_prompt: format!("变更分析：\n{}\n项目上下文：\n{}", diff, context),
            temperature: 0.7,
            max_tokens: 50,
        }
    }

    /// 为代码质量审查优化
    pub fn for_code_review(code: &str) -> Self {
        Self {
            model: ModelProvider::Claude35,
            system_prompt: "分析代码质量，专注安全、性能和可维护性，Markdown格式输出".to_string(),
            user_prompt: format!("代码质量分析:\n{}", code),
            temperature: 0.3,
            max_tokens: 800,
        }
    }

    /// 为Changelog生成优化
    pub fn for_changelog(changelog_data: &str) -> Self {
        Self {
            model: ModelProvider::Claude35,
            system_prompt: "生成符合keepachangelog标准的变更日志，Markdown格式".to_string(),
            user_prompt: format!("变更日志生成:\n{}", changelog_data),
            temperature: 0.5,
            max_tokens: 1000,
        }
    }
}

/// GXL AI原生响应结构
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AiResponse {
    pub content: String,
    pub model: ModelProvider,
    pub input_tokens: u64,
    pub output_tokens: u64,
    pub confidence: f32,
}

impl AiResponse {
    /// 提取commit消息专用
+    pub fn as_commit_message(&self) -> String {
++        self.content.trim().lines().next().unwrap_or(&self.content).to_string()
++    }
++
++    /// 验证commit格式是否符合conventional commits
++    pub fn validate_commit(&self) -> bool {
++        let lines: Vec<&str> = self.content.lines().collect();
++        if lines.is_empty() { return false; }
++
++        let first_line = lines[0].trim();
++        // conventional commit格式: type(scope): 描述
++        first_line.split(':').count() >= 2
++    }
++}
++
