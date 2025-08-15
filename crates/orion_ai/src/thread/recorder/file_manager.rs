use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use chrono::{DateTime, Utc};
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use super::SummaryExtractor;
use crate::config::ThreadConfig;
use crate::error::{AiErrReason, AiError, AiResult};
use crate::provider::{AiRequest, AiResponse};

/// Thread文件管理器，负责记录交互到文件
pub struct ThreadFileManager {
    config: std::sync::Arc<ThreadConfig>,
    interaction_counter: AtomicUsize, // 交互计数器
    base_path: PathBuf,               // 基础路径
}

impl ThreadFileManager {
    pub fn new(config: ThreadConfig) -> Self {
        let base_path = Self::resolve_storage_path(&config.storage_path);

        Self {
            config: std::sync::Arc::new(config),
            interaction_counter: AtomicUsize::new(1),
            base_path,
        }
    }

    /// 记录一次AI交互
    pub async fn record_interaction(
        &self,
        timestamp: DateTime<Utc>,
        request: &AiRequest,
        response: &AiResponse,
    ) -> AiResult<()> {
        // 1. 生成今日文件路径
        let file_path = self.generate_daily_file_path(&timestamp);

        // 2. 确保目录存在
        self.ensure_directory_exists(&file_path)?;

        // 3. 提取总结性内容
        let summary_content = self.extract_summary_content(&response.content);

        // 4. 格式化记录内容
        let interaction_number = self.interaction_counter.fetch_add(1, Ordering::SeqCst);
        let record_content = self.format_interaction_record(
            timestamp,
            interaction_number,
            request,
            &summary_content,
        );

        // 5. 追加写入文件
        self.append_to_file(&file_path, &record_content).await
    }

    /// 解析存储路径中的环境变量
    fn resolve_storage_path(storage_path: &Path) -> PathBuf {
        let path_str = storage_path.to_string_lossy();

        // 简单的环境变量替换
        if let Some(env_var) = path_str.strip_prefix('$') {
            if let Ok(env_value) = std::env::var(env_var) {
                return PathBuf::from(env_value);
            }
        }

        storage_path.to_path_buf()
    }

    /// 生成每日文件路径
    fn generate_daily_file_path(&self, timestamp: &DateTime<Utc>) -> PathBuf {
        let date_str = timestamp.format("%Y-%m-%d").to_string();
        let filename = self
            .config
            .filename_template
            .replace("YYYY-MM-DD", &date_str);

        // 确保文件名以.md结尾
        let filename = if filename.ends_with(".md") {
            filename
        } else {
            format!("{filename}.md")
        };

        self.base_path.join(filename)
    }

    /// 确保目录存在
    fn ensure_directory_exists(&self, file_path: &Path) -> AiResult<()> {
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AiError::from(AiErrReason::ContextError(format!(
                    "Failed to create directory {}: {}",
                    parent.display(),
                    e
                )))
            })?;
        }
        Ok(())
    }

    /// 提取总结性内容
    fn extract_summary_content(&self, content: &str) -> String {
        let extractor = SummaryExtractor::new(&self.config.summary_keywords);
        extractor.extract_with_length_limits(
            content,
            self.config.min_summary_length,
            self.config.max_summary_length,
        )
    }

    /// 格式化交互记录
    fn format_interaction_record(
        &self,
        timestamp: DateTime<Utc>,
        interaction_number: usize,
        request: &AiRequest,
        summary_content: &str,
    ) -> String {
        let role_str = request
            .role
            .as_ref()
            .map_or("None".to_string(), |r| r.to_string());

        format!(
            "## 交互记录 {}\n**时间**: {}\n**模型**: {}\n**角色**: {}\n\n### 用户请求\n```text\n{}```\n\n### AI响应（总结）\n{}\n\n",
            interaction_number,
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            request.model,
            role_str,
            request.user_prompt,
            summary_content
        )
    }

    /// 异步追加内容到文件
    async fn append_to_file(&self, path: &Path, content: &str) -> AiResult<()> {
        let file_exists = path.exists();

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|e| {
                AiError::from(AiErrReason::ContextError(format!(
                    "Failed to open file {}: {}",
                    path.display(),
                    e
                )))
            })?;

        // 如果文件是刚创建的，添加标题
        if !file_exists {
            let date_str = chrono::Utc::now().format("%Y-%m-%d").to_string();
            let title = format!("# Thread记录 - {date_str}\n\n");
            file.write_all(title.as_bytes()).await.map_err(|e| {
                AiError::from(AiErrReason::ContextError(format!(
                    "Failed to write title to file {}: {}",
                    path.display(),
                    e
                )))
            })?;
        }

        file.write_all(content.as_bytes()).await.map_err(|e| {
            AiError::from(AiErrReason::ContextError(format!(
                "Failed to write to file {}: {}",
                path.display(),
                e
            )))
        })?;

        file.flush().await.map_err(|e| {
            AiError::from(AiErrReason::ContextError(format!(
                "Failed to flush file {}: {}",
                path.display(),
                e
            )))
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_file_manager_basic() {
        let temp_dir = TempDir::new().unwrap();
        let storage_path = temp_dir.path();

        let config = ThreadConfig {
            enabled: true,
            storage_path: storage_path.to_path_buf(),
            filename_template: "test-YYYY-MM-DD.md".to_string(),
            min_summary_length: 10,
            max_summary_length: 50,
            summary_keywords: vec!["总结".to_string()],
            inform_ai: false,
            inform_message: "".to_string(),
        };

        let file_manager = ThreadFileManager::new(config);
        let timestamp = Utc::now();

        let request = AiRequest::builder()
            .model("test-model")
            .system_prompt("test system".to_string())
            .user_prompt("test user".to_string())
            .build();

        let response = AiResponse {
            content: "这是一个测试响应。总结：测试成功。".to_string(),
            model: "test-model".to_string(),
            usage: crate::provider::UsageInfo {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
                estimated_cost: None,
            },
            finish_reason: None,
            provider: crate::provider::AiProviderType::Mock,
            metadata: std::collections::HashMap::new(),
        };

        // 记录交互
        let result = file_manager
            .record_interaction(timestamp, &request, &response)
            .await;
        assert!(result.is_ok());

        // 检查文件是否创建
        let date_str = timestamp.format("%Y-%m-%d").to_string();
        let expected_file = storage_path.join(format!("test-{date_str}.md"));
        assert!(expected_file.exists());

        // 检查文件内容
        let content = std::fs::read_to_string(&expected_file).unwrap();
        assert!(content.contains("## 交互记录 1"));
        assert!(content.contains("**模型**: test-model"));
        // 检查是否包含总结内容，不要求完全匹配
        assert!(content.contains("总结") || content.contains("测试成功"));
    }
}
