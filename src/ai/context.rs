use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::ai::error::{AiErrReason, AiResult};

/// Git上下文信息收集
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
    pub languages: HashMap<String, usize>,
}

/// Git变更信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitChange {
    pub file_path: PathBuf,
    pub change_type: ChangeType,
}

/// 变更类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed(String),
    Copied(String),
}

/// Git提交历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitCommit {
    pub hash: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub message: String,
}

impl GitContext {
    /// 从当前目录收集Git上下文
    pub fn from_current_dir() -> AiResult<Option<GitContext>> {
        let current_dir =
            std::env::current_dir().map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if !Self::is_git_repository(&current_dir) {
            return Ok(None);
        }

        let branch = Self::get_current_branch(&current_dir)?;
        let staged_changes = Self::get_staged_changes(&current_dir)?;
        let unstaged_changes = Self::get_unstaged_changes(&current_dir)?;
        let diff_content = Self::get_staged_diff(&current_dir)?;
        let commit_history = Self::get_recent_commits(&current_dir)?;
        let languages = Self::analyze_languages(&current_dir)?;

        let total_files_changed = staged_changes.len() + unstaged_changes.len();

        Ok(Some(GitContext {
            repository_path: current_dir,
            branch,
            is_dirty: !staged_changes.is_empty() || !unstaged_changes.is_empty(),
            staged_changes,
            unstaged_changes,
            total_files_changed,
            diff_content,
            commit_history,
            languages,
        }))
    }

    /// 检查是否为git仓库
    fn is_git_repository<P: AsRef<Path>>(path: P) -> bool {
        Command::new("git")
            .arg("rev-parse")
            .arg("--git-dir")
            .current_dir(path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// 获取当前分支
    fn get_current_branch<P: AsRef<Path>>(repo_path: P) -> AiResult<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "--abbrev-ref", "HEAD"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Ok("main".to_string())
        }
    }

    /// 获取已暂存变更
    fn get_staged_changes<P: AsRef<Path>>(repo_path: P) -> AiResult<Vec<GitChange>> {
        Self::get_git_changes(repo_path, true)
    }

    /// 获取未暂存变更
    fn get_unstaged_changes<P: AsRef<Path>>(repo_path: P) -> AiResult<Vec<GitChange>> {
        Self::get_git_changes(repo_path, false)
    }

    /// 获取git变更
    fn get_git_changes<P: AsRef<Path>>(repo_path: P, staged: bool) -> AiResult<Vec<GitChange>> {
        let mut cmd = Command::new("git");
        let mut args = vec!["diff", "--name-only"];

        if staged {
            args.push("--cached");
        }

        cmd.args(&args);
        cmd.current_dir(&repo_path);

        let output = cmd
            .output()
            .map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let files: Vec<&str> = stdout.lines().collect();

        let mut changes = Vec::new();
        for file_path in files {
            let file_path = file_path.trim();
            if !file_path.is_empty() {
                changes.push(GitChange {
                    file_path: repo_path.as_ref().join(file_path),
                    change_type: ChangeType::Modified,
                });
            }
        }
        Ok(changes)
    }

    /// 获取已暂存的diff内容
    fn get_staged_diff<P: AsRef<Path>>(repo_path: P) -> AiResult<String> {
        let output = Command::new("git")
            .args(&["diff", "--cached", "--no-color", "--no-ext-diff"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).to_string())
        } else {
            Ok("".to_string())
        }
    }

    /// 获取最近的提交历史
    fn get_recent_commits<P: AsRef<Path>>(repo_path: P) -> AiResult<Vec<GitCommit>> {
        let output = Command::new("git")
            .args(&["log", "--oneline", "-10"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut commits = Vec::new();

        for line in stdout.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }

            let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();
            if parts.len() >= 2 {
                commits.push(GitCommit {
                    hash: parts[0].to_string(),
                    author: "unknown".to_string(),
                    email: "unknown".to_string(),
                    date: "unknown".to_string(),
                    message: parts[1].to_string(),
                });
            }
        }

        Ok(commits)
    }

    /// 分析项目语言组成
    fn analyze_languages<P: AsRef<Path>>(repo_path: P) -> AiResult<HashMap<String, usize>> {
        let mut stats = HashMap::new();

        let output = Command::new("git")
            .arg("ls-files")
            .current_dir(repo_path)
            .output()
            .map_err(|e| AiErrReason::ContextError(e.to_string()))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            for file in stdout.lines() {
                let path = Path::new(file);
                let extension = path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .unwrap_or("unknown")
                    .to_lowercase();

                *stats.entry(extension).or_insert(0) += 1;
            }
        }

        Ok(stats)
    }
}
