use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::ai::error::{AiError, AiResult};

/// 统一的AI能力枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiCapability {
    Analyze,        // 代码理解分析
    Suggest,        // 基于上下文的建议
    Check,          // 问题检测和审查
    Generate,       // 代码/文档创建
    Refactor,       // 重构建议
    Deploy,         // 智能部署决策
}

/// AI提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AiProviderType {
    OpenAi,
    Anthropic,
    Ollama,
    Mock,
}

impl std::fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProviderType::OpenAi => write!(f, "openai"),
            AiProviderType::Anthropic => write!(f, "anthropic"),
            AiProviderType::Ollama => write!(f, "ollama"),
            AiProviderType::Mock => write!(f, "mock"),
        }
    }
}

/// 模型信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub name: String,
    pub provider: AiProviderType,
    pub max_tokens: usize,
    pub supports_images: bool,
    pub supports_reasoning: bool,
    pub cost_per_1k_input: f64,  // 美元
    pub cost_per_1k_output: f64, // 美元
}

impl ModelInfo {
    pub fn is_compatible(&self, capability: AiCapability) -> bool {
        match capability {
            AiCapability::Analyze | AiCapability::Check => true,
            AiCapability::Suggest => true,
            AiCapability::Generate => true,
            AiCapability::Refactor => true,
            AiCapability::Deploy => true,
        }
    }
}

/// 统一AI请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub capabilities: Vec<AiCapability>,
    pub context: HashMap<String, String>,
    pub sensitive_filter: bool,
}

impl AiRequest {
    pub fn builder() -> AiRequestBuilder {
        AiRequestBuilder::new()
    }
}

pub struct AiRequestBuilder {
    model: String,
    system_prompt: String,
    user_prompt: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    capabilities: Vec<AiCapability>,
    context: HashMap<String, String>,
    sensitive_filter: bool,
}

impl AiRequestBuilder {
    pub fn new() -> Self {
        Self {
            model: "gpt-4o".to_string(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: None,
            temperature: Some(0.7),
            capabilities: vec![],
            context: HashMap::new(),
            sensitive_filter: true,
        }
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = model.into();
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = prompt.into();
        self
    }

    pub fn user_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.user_prompt = prompt.into();
        self
    }

    pub fn max_tokens(mut self, tokens: usize) -> Self {
        self.max_tokens = Some(tokens);
        self
    }

    pub fn temperature(mut self, temp: f32) -> Self {
        self.temperature = Some(temp);
        self
    }

    pub fn capability(mut self, cap: AiCapability) -> Self {
        self.capabilities.push(cap);
        self
    }

    pub fn context(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.insert(key.into(), value.into());
        self
    }

    pub fn sensitive_filter(mut self, filter: bool) -> Self {
        self.sensitive_filter = filter;
        self
    }

    pub fn build(self) -> AiRequest {
        AiRequest {
            model: self.model,
            system_prompt: self.system_prompt,
            user_prompt: self.user_prompt,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            capabilities: self.capabilities,
            context: self.context,
            sensitive_filter: self.sensitive_filter,
        }
    }
}

/// 统一AI响应结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiResponse {
    pub content: String,
    pub model: String,
    pub usage: UsageInfo,
    pub finish_reason: Option<String>,
    pub provider: AiProviderType,
    pub metadata: HashMap<String, serde_json::Value>,
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct UsageInfo {
+    pub prompt_tokens: usize,
+    pub completion_tokens: usize,
+    pub total_tokens: usize,
+    pub estimated_cost: Option<f64>,
+}
+
+/// AI提供商trait定义
+#[async_trait]
+pub trait AiProvider: Send + Sync {
+    /// 获取提供商类型
+    fn provider_type(&self) -> AiProviderType;
+
+    /// 检查模型可用性
+    async fn is_model_available(&self, model: &str) -> bool;
+
+    /// 获取可用模型列表
+    async fn list_models(&self) -> AiResult<Vec<ModelInfo>>;
+
+    /// 发送AI请求
+    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse>;
+
+    /// 获取配置参数
+    fn get_config_keys(&self) -> Vec<&'static str> {
+        vec![]
+    }
+
+    /// 健康检查
+    async fn health_check(&self) -> AiResult<bool> {
+        self.list_models().await.map(|_| true)
+    }
+
+    /// 计算预估成本
+    fn estimate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> Option<f64>;
+
+    /// 检查token限制
+    fn check_token_limit(&self, model: &str, max_tokens: usize) -> bool;
+}
