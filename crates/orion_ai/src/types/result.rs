use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::provider::FunctionResult;

/// 简化的执行结果类型
///
/// 这个结构体提供了简化的AI执行结果封装，复用orion_ai的FunctionResult，
/// 只包含必要的字段，避免过度复杂化。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// AI响应的主要内容
    pub content: String,
    /// 工具调用结果列表
    pub tool_calls: Vec<FunctionResult>,
    /// 执行时间戳
    pub timestamp: DateTime<Utc>,
    /// 执行状态
    pub status: ExecutionStatus,
    /// 额外的元数据
    pub metadata: HashMap<String, String>,
}

/// 执行状态枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// 执行成功
    Success,
    /// 执行失败
    Failed,
    /// 部分成功（有些工具调用成功，有些失败）
    Partial,
    /// 超时
    Timeout,
    /// 已取消
    Cancelled,
}

impl Default for ExecutionResult {
    fn default() -> Self {
        Self::new(String::new())
    }
}

impl ExecutionResult {
    /// 创建新的执行结果
    pub fn new(content: String) -> Self {
        Self {
            content,
            tool_calls: Vec::new(),
            timestamp: Utc::now(),
            status: ExecutionStatus::Success,
            metadata: HashMap::new(),
        }
    }

    /// 设置工具调用结果
    pub fn with_tool_calls(mut self, tool_calls: Vec<FunctionResult>) -> Self {
        self.tool_calls = tool_calls;
        // 根据工具调用结果更新状态
        self.update_status_from_tool_calls();
        self
    }

    /// 设置执行状态
    pub fn with_status(mut self, status: ExecutionStatus) -> Self {
        self.status = status;
        self
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 判断是否成功
    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }

    /// 判断是否失败
    pub fn is_failed(&self) -> bool {
        matches!(self.status, ExecutionStatus::Failed)
    }

    /// 获取成功工具调用的数量
    pub fn successful_tool_calls_count(&self) -> usize {
        self.tool_calls
            .iter()
            .filter(|call| call.error.is_none())
            .count()
    }

    /// 获取失败工具调用的数量
    pub fn failed_tool_calls_count(&self) -> usize {
        self.tool_calls
            .iter()
            .filter(|call| call.error.is_some())
            .count()
    }

    /// 根据工具调用结果更新状态
    fn update_status_from_tool_calls(&mut self) {
        if self.tool_calls.is_empty() {
            self.status = ExecutionStatus::Success;
            return;
        }

        let success_count = self.successful_tool_calls_count();
        let failed_count = self.failed_tool_calls_count();

        if success_count == 0 && failed_count > 0 {
            self.status = ExecutionStatus::Failed;
        } else if failed_count > 0 {
            self.status = ExecutionStatus::Partial;
        } else {
            self.status = ExecutionStatus::Success;
        }
    }

    /// 获取执行耗时（秒）
    pub fn duration_since(&self) -> f64 {
        let now = Utc::now();
        now.signed_duration_since(self.timestamp).num_seconds() as f64
    }

    /// 获取格式化的时间戳
    pub fn formatted_timestamp(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
    }

    /// 获取执行摘要
    pub fn summary(&self) -> String {
        format!(
            "Status: {:?}, Tool calls: {} ({} success, {} failed), Content length: {}",
            self.status,
            self.tool_calls.len(),
            self.successful_tool_calls_count(),
            self.failed_tool_calls_count(),
            self.content.len()
        )
    }
}

impl From<ExecutionResult> for serde_json::Value {
    fn from(result: ExecutionResult) -> Self {
        serde_json::json!({
            "content": result.content,
            "tool_calls": result.tool_calls,
            "timestamp": result.timestamp.to_rfc3339(),
            "status": format!("{:?}", result.status),
            "metadata": result.metadata,
        })
    }
}

/// 执行结果的构造器
pub struct ExecutionResultBuilder {
    content: String,
    tool_calls: Vec<FunctionResult>,
    timestamp: Option<DateTime<Utc>>,
    status: Option<ExecutionStatus>,
    metadata: HashMap<String, String>,
}

impl ExecutionResultBuilder {
    /// 创建新的构造器
    pub fn new() -> Self {
        Self {
            content: String::new(),
            tool_calls: Vec::new(),
            timestamp: None,
            status: None,
            metadata: HashMap::new(),
        }
    }

    /// 设置内容
    pub fn content(mut self, content: String) -> Self {
        self.content = content;
        self
    }

    /// 添加工具调用结果
    pub fn add_tool_call(mut self, tool_call: FunctionResult) -> Self {
        self.tool_calls.push(tool_call);
        self
    }

    /// 设置工具调用结果列表
    pub fn tool_calls(mut self, tool_calls: Vec<FunctionResult>) -> Self {
        self.tool_calls = tool_calls;
        self
    }

    /// 设置时间戳
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// 设置状态
    pub fn status(mut self, status: ExecutionStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// 添加元数据
    pub fn metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// 构建ExecutionResult
    pub fn build(self) -> ExecutionResult {
        let mut result = ExecutionResult::new(self.content)
            .with_tool_calls(self.tool_calls)
            .with_status(self.status.unwrap_or(ExecutionStatus::Success));

        // 添加元数据
        for (key, value) in self.metadata {
            result = result.with_metadata(key, value);
        }

        // 设置时间戳（如果提供）
        if let Some(timestamp) = self.timestamp {
            result.timestamp = timestamp;
        }

        result
    }
}

impl Default for ExecutionResultBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::provider::FunctionResult;

    #[test]
    fn test_execution_result_creation() {
        let result = ExecutionResult::new("test content".to_string());

        assert_eq!(result.content, "test content");
        assert!(result.tool_calls.is_empty());
        assert!(result.is_success());
        assert!(!result.is_failed());
    }

    #[test]
    fn test_execution_result_with_tool_calls() {
        let tool_call1 = FunctionResult {
            name: "test_tool".to_string(),
            result: serde_json::json!({"success": true}),
            error: None,
        };

        let tool_call2 = FunctionResult {
            name: "failed_tool".to_string(),
            result: serde_json::json!({}),
            error: Some("test error".to_string()),
        };

        let result = ExecutionResult::new("test".to_string())
            .with_tool_calls(vec![tool_call1.clone(), tool_call2]);

        assert_eq!(result.tool_calls.len(), 2);
        assert_eq!(result.successful_tool_calls_count(), 1);
        assert_eq!(result.failed_tool_calls_count(), 1);
        assert_eq!(result.status, ExecutionStatus::Partial);
    }

    #[test]
    fn test_execution_result_builder() {
        let tool_call = FunctionResult {
            name: "test_tool".to_string(),
            result: serde_json::json!({"success": true}),
            error: None,
        };

        let result = ExecutionResultBuilder::new()
            .content("test content".to_string())
            .add_tool_call(tool_call)
            .status(ExecutionStatus::Success)
            .metadata("test_key".to_string(), "test_value".to_string())
            .build();

        assert_eq!(result.content, "test content");
        assert_eq!(result.tool_calls.len(), 1);
        assert_eq!(result.status, ExecutionStatus::Success);
        assert_eq!(
            result.metadata.get("test_key"),
            Some(&"test_value".to_string())
        );
    }

    #[test]
    fn test_execution_result_summary() {
        let tool_call1 = FunctionResult {
            name: "success_tool".to_string(),
            result: serde_json::json!({"success": true}),
            error: None,
        };

        let tool_call2 = FunctionResult {
            name: "failed_tool".to_string(),
            result: serde_json::json!({}),
            error: Some("error".to_string()),
        };

        let result =
            ExecutionResult::new("test".to_string()).with_tool_calls(vec![tool_call1, tool_call2]);

        let summary = result.summary();
        assert!(summary.contains("Status: Partial"));
        assert!(summary.contains("Tool calls: 2"));
        assert!(summary.contains("1 success"));
        assert!(summary.contains("1 failed"));
    }

    #[test]
    fn test_execution_result_to_json() {
        let result = ExecutionResult::new("test content".to_string());
        let json_value: serde_json::Value = result.into();

        assert_eq!(json_value["content"], "test content");
        assert!(json_value["tool_calls"].is_array());
        assert!(json_value["timestamp"].is_string());
        assert_eq!(json_value["status"], "Success");
    }
}
