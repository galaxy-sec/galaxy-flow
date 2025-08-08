use crate::ai::{AiClient, AiResult};

pub struct SmartCommitFlow {
    client: AiClient,
    auto_mode: bool,
}

impl SmartCommitFlow {
    pub fn new(client: AiClient, auto_mode: bool) -> Self {
        Self { client, auto_mode }
    }

    pub async fn execute(&self) -> AiResult<()> {
        // 1. 收集Git上下文
        let repo_path = std::env::current_dir()
            .map_err(|e| crate::ai::error::AiError::ContextError(e.to_string()))?;

        let status = std::process::Command::new("git")
            .args(&["status", "--porcelain"])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| crate::ai::error::AiError::ContextError(e.to_string()))?;

        if !status.status.success() {
            println!("❌ Current directory is not a Git repository");
            return Ok(());
        }

        let changed_files = String::from_utf8_lossy(&status.stdout);
        if changed_files.trim().is_empty() {
            println!("🎉 No changes to commit");
            return Ok(());
        }

        // 2. 获取提交的diff
        let diff_output = std::process::Command::new("git")
            .args(&["diff", "--cached", "--no-color"])
            .current_dir(&repo_path)
            .output()
            .map_err(|e| crate::ai::error::AiError::ContextError(e.to_string()))?;

        let diff_content = String::from_utf8_lossy(&diff_output.stdout).to_string();

        // 2. 准备提交上下文
        let context = self.format_context(&changed_files, &diff_content);

        // 3. 生成提交信息
        let response = self.client.smart_commit(&context).await?;
        let mut commit_msg = response.content.trim().to_string();

        // 4. 交互确认
        if !self.auto_mode {
            commit_msg = self.confirm_commit(&commit_msg)?;
        }

        // 5. 执行Git操作
        self.perform_git_commit(&git_context, &commit_msg)?;

        println!("✅ AI智能提交完成: {}", commit_msg);
        Ok(())
    }

    fn format_context(&self, changed_files: &str, diff_content: &str) -> String {
        let mut output = String::new();

        let current_branch = String::from_utf8_lossy(
            &std::process::Command::new("git")
                .args(&["branch", "--show-current"])
                .output()
                .unwrap()
                .stdout
        ).trim().to_string();

        output.push_str(&format!("# 分支: {}\n", current_branch));
        output.push_str(&format!("# 文件变更:\n{}", changed_files));

        if !diff_content.is_empty() {
            output.push_str("\n# 代码变更:\n");
            output.push_str(diff_content);
        }

        output
    }

    fn confirm_commit(&self, message: &str) -> AiResult<String> {
        println!("🤖 AI建议的提交信息：");
        println!("📝 {}", message);

        loop {
            println!("接受(y)或编辑(e)或取消(c)？ [y/e/c]: ");

            let mut input = String::new();
            std::io::stdin().read_line(&mut input).unwrap();

            match input.trim().to_lowercase().as_str() {
                "y" | "yes" => return Ok(message.to_string()),
                "e" | "edit" => return self.edit_commit(message),
                "c" | "cancel" => return Ok("".to_string()),
                _ => println!("请输入 y, e, 或 c"),
            }
        }
    }

    fn edit_commit(&self, original: &str) -> AiResult<String> {
        use std::io::{Write, stdin};

        println!("📝 编辑提交信息 (留空使用原信息):");
        println!("原信息: {}", original);

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();

        let edited = input.trim();
        if edited.is_empty() {
            Ok(original.to_string())
        } else {
            Ok(edited.to_string())
        }
    }

    fn perform_git_commit(&self, message: &str) -> AiResult<()> {
        if message.trim().is_empty() {
            return Ok(());
        }

        // 执行提交
        let output = std::process::Command::new("git")
            .args(&["commit", "-m", message])
            .output()
            .map_err(|e| crate::ai::error::AiError::ContextError(e.to_string()))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(crate::ai::error::AiError::ContextError(format!(
                "Git commit failed: {}", error
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
+    use super::*;
+
+    #[test]
+    fn test_format_context() {
+        let
