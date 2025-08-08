use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use serde::{Deserialize, Serialize};

use crate::ai::error::{AiError, AiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub repository_path: PathBuf,
    pub branch: String,
    pub is_dirty: bool,
    pub staged_changes: Vec<GitChange>,
    pub unstaged_changes: Vec<GitChange>,
    pub total_files_changed: usize,
    pub diff_content: String,
    pub commit_history: Vec<GitCommit>,
    pub languages: HashMap<String, usize>, // 语言统计
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
    pub insertions: usize,
    pub deletions: usize,
    pub is_binary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed(String), // 新文件名
    Copied(String),  // 原始文件名
    TypeChanged,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
    pub body: Option<String>,
}

impl GitContext {
    /// 从当前目录收集完整的Git上下文
    pub fn from_current_dir() -> AiResult<Option<GitContext>> {
        let current_dir = std::env::current_dir()
            .map_err(|e| AiError::ContextError(e.to_string()))?;

        // 检查是否是git仓库
        if !Self::is_git_repository(&current_dir) {
            return Ok(None);
        }

        let branch = Self::get_current_branch(&current_dir)?;
        let staged_changes = Self::get_staged_changes(&current_dir)?;
        let unstaged_changes = Self::get_unstaged_changes(&current_dir)?;
        let diff = Self::get_staged_diff(&current_dir)?;
        let history = Self::get_recent_commits(&current_dir, 5)?;
        let languages = Self::analyze_languages(&current_dir)?;

        let total_files_changed = staged_changes.len() + unstaged_changes.len();

        Ok(Some(GitContext {
            repository_path: current_dir,
            branch,
            is_dirty: !staged_changes.is_empty() || !unstaged_changes.is_empty(),
            staged_changes,
            unstaged_changes,
            total_files_changed,
            diff_content: diff,
            commit_history: history,
            languages,
        }))
    }

    fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        let git_check = Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();

        matches!(git_check, Ok(status) if status.success())
    }

    fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> AiResult<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiError::ContextError(e.to_string()))?;

        if output.status.success() {
            let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
            Ok(branch)
        } else {
            Err(AiError::ContextError("Failed to get current branch".to_string()))
        }
    }

    fn get_staged_changes<P: AsRef<Path>>(repo_path: P) -> AiResult<Vec<GitChange>> {
        Self::get_git_changes(repo_path, true)
    }

    fn get_unstaged_changes<P: AsRef<Path>>(repo_path: P) -> AiResult<Vec<GitChange>> {
        Self::get_git_changes(repo_path, false)
    }

    fn get_git_changes<P: AsRef<Path>>(repo_path: P, staged: bool) -> AiResult<Vec<GitChange>> {
        let mut cmd = Command::new("git");

        if staged {
            cmd.args(&["diff", "--cached", "--numstat", "--no-renames"]);
        } else {
            cmd.args(&["diff", "--numstat", "--no-renames"]);
        }

        let output = cmd
            .current_dir(&repo_path)
            .output()
            .map_err(|e| AiError::ContextError(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut changes = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.trim().split_whitespace().collect();
            if parts.len() >= 3 {
                let is_binary = parts.len() > 3 && parts[2] == "-";
                changes.push(GitChange {
                    file_path: repo_path.as_ref().join(parts[2]),
                    change_type: ChangeType::Modified,
                    insertions: parts[0].parse().unwrap_or(0),
                    deletions: parts[1].parse().unwrap_or(0),
                    is_binary,
                });
            }
        }

        Ok(changes)
    }

    fn get_staged_diff<P: AsRef<Path>>(repo_path: P) -> AiResult<String> {
        let output = Command::new("git")
            .args(&["diff", "--cached", "--no-color", "--no-ext-diff"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiError::ContextError(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok("".to_string())
        }
    }

    fn get_recent_commits<P: AsRef<Path>>(repo_path: P, limit: usize) -> AiResult<Vec<GitCommit>> {
        let output = Command::new("git")
            .args(&[
                "log",
+                "--pretty=format:%h|%an|%ae|%ad|%s|%b",
+                "--date=short",
+                "--max-count=",
+                &limit.to_string(),
+            ])
+            .current_dir(repo_path)
+            .output()
+            .map_err(|e| AiError::ContextError(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('|').collect();
            if parts.len() >= 5 {
                commits.push(GitCommit {
+                    hash: parts[0].to_string(),
+                    author: parts[1].to_string(),
+                    email: parts[2].to_string(),
+                    date: parts[3].to_string(),
+                    message: parts[4].to_string(),
+                    body: if parts.len() > 5 { Some(parts[5].to_string()) } else { None },
+                });
+            }
+        }

        Ok(commits)
+    }

+    fn analyze_languages<P: AsRef<Path>>(repo_path: P) -> AiResult<HashMap<String, usize>> {
+        let mut stats = HashMap::new();
+
+        // 使用git简单统计文件类型
+        let output = Command::new("git")
+            .args(&["ls-files"])
+            .current_dir(repo_path)
+            .output()
+            .map_err(|e| AiError::ContextError(e.to_string()))?;

+        if output.status.success() {
+            let stdout = String::from_utf8_lossy(&output.stdout);
+            for file in stdout.lines() {
+                let extension = Path::new(file)
+                    .extension()
+                    .and_then(|ext| ext.to_str())
+                    .unwrap_or("unknown");
+
+                *stats.entry(extension.to_string()).or_insert(0) += 1;
+            }
+        }

+        Ok(stats)
    }
 }
