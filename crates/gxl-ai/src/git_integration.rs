use crate::error::{AiError, Result};
use git2::{Diff, DiffFormat, Repository};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tokio::fs;

/// GXL特色Git上下文分析系统
pub struct GitContext {
    repo: Repository,
    path: String,
}

impl GitContext {
    /// 创建新的GXL Git上下文
    pub fn new() -> Result<Self> {
        let current_dir = std::env::current_dir()
            .expect("Failed to get current directory");

        let repo = Repository::discover(&current_dir)
            .map_err(|e| AiError::GitError(e.to_string()))?;

        Ok(Self {
            repo,
            path: current_dir.to_string_lossy().to_string(),
        })
    }

    /// 获取已暂存变更的完整上下文
    pub async fn get_staged_changes(&self, limit: usize) -> Result<ChangeContext> {
        let head = self.repo.head().ok();
        let head_tree = head.as_ref().and_then(|h| h.peel_to_tree().ok());

        let mut index = self.repo.index()
            .map_err(|e| AiError::GitError(e.to_string()))?;

        let diff = self.repo.diff_tree_to_index(
+            head_tree.as_ref(),
++            Some(&index),
++            None
++        ).map_err(|e| AiError::GitError(e.to_string()))?;
++
++        let mut diff_text = String::new();
++        diff.print(DiffFormat::Patch, |delta, _hunk, line| {
++            let content = String::from_utf8_lossy(line.content());
++            diff_text.push_str(&format!("{}{}", {
++                match line.new_lineno() {
++                    Some(_) => "+",
++                    None if line.old_lineno().is_some() => "-",
++                    _ => " ",
++                }
++            }, content));
++            true
++        }).map_err(|e| AiError::GitError(e.to_string()))?;
++
++        // 限制上下文长度
++        if diff_text.len() > limit * 5000 {
++            diff_text = diff_text.chars().take(limit * 5000).collect();
++            diff_text.push_str("\n... (内容截断)");
++        }
++
++        Ok(ChangeContext {
++            files: Self::extract_changed_files(&diff)?,
++            diff_text,
++            type_classification: Self::classify_changes(&diff),
++        })
++    }
++
++    /// GXL特色：变
++    fn extract_changed_files(diff: &Diff) -> Result<Vec<String>> {
++        let mut files = Vec::new();
++        for delta in diff.deltas() {
++            match delta.status() {
++                git2::Delta::Added |
++                git2::Delta::Modified |
++                git2::Delta::Deleted => {
++                    if let Some(new_file) = delta.new_file().path() {
++                        files.push(new_file.to_string_lossy().to_string());
++                    }
++                },
++                _ => continue,
++            }
++        }
++        Ok(files)
++    }
++
++    /// GXL特色变更分类算法
++    fn classify_changes(diff: &Diff) -> HashMap<String, Vec<String>> {
++        let mut classification = HashMap::new();
++
++        for delta in diff.deltas() {
++            let path = delta.new_file().path()
++                .map(|p| p.to_string_lossy().to_string())
++                .unwrap_or_else(|| "unknown".to_string());
++
++            let status = match delta.status() {
++                git2::Delta::Added => "feat",
++                git2::Delta::Modified => "fix",
++                git2::Delta::Deleted => "remove",
++                git2::Delta::Renamed => "refactor",
++                git2::Delta::Copied => "chore",
++                _ => "update",
++            };
++
++            classification.entry(status.to_string())
++                .or_insert_with(Vec::new)
++                .push(path);
++        }
++
++        classification
++    }
++
++    /// 获取项目上下文JSON格式
++    pub async fn generate_context_json(&self) -> String {
++        let repo_info = self.get_repository_info().await;
++        let recent_commits = self.get_recent_commits(5).await;
++
++        json!({
++            "repository": repo_info,
++            "recent_activity": recent_commits,
++            "file_structure": self.get_file_patterns().await
++        }).to_string()
++    }
++
++    /// GXL特色：代码上下文理解
++    pub async fn get_code_context(&self, files
