use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::ai::error::AiResult;

use super::capabilities::AiDevCapability;

/// AI提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

impl From<AiProviderType> for &'static str {
    fn from(provider: AiProviderType) -> Self {
        match provider {
            AiProviderType::OpenAi => "openai",
            AiProviderType::Anthropic => "anthropic",
            AiProviderType::Ollama => "ollama",
            AiProviderType::Mock => "mock",
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
    pub fn is_compatible(&self, capability: AiDevCapability) -> bool {
        match capability {
            AiDevCapability::Analyze | AiDevCapability::Check => true,
            AiDevCapability::Suggest => true,
            AiDevCapability::Generate => true,
            AiDevCapability::Refactor => true,
            AiDevCapability::Deploy => true,
            AiDevCapability::Commit => true,
            AiDevCapability::Review => true,
            AiDevCapability::Understand => true,
            AiDevCapability::Predict => true,
            AiDevCapability::Collaborate => true,
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
    pub capability: AiDevCapability,
}

impl AiRequest {
    pub fn builder() -> AiRequestBuilder {
        AiRequestBuilder::new()
    }
}

/// AI请求构建器
pub struct AiRequestBuilder {
    model: String,
    system_prompt: String,
    user_prompt: String,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    capability: AiDevCapability,
}

impl AiRequestBuilder {
    pub fn new() -> Self {
        Self {
            model: "gpt-4o-mini".to_string(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: None,
            temperature: Some(0.7),
            capability: AiDevCapability::Analyze,
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

    pub fn capability(mut self, cap: AiDevCapability) -> Self {
        self.capability = cap;
        self
    }

    pub fn build(self) -> AiRequest {
        AiRequest {
            model: self.model,
            system_prompt: self.system_prompt,
            user_prompt: self.user_prompt,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            capability: self.capability,
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
    pub estimated_cost: Option<f64>,
}

/// AI提供商trait定义
#[async_trait]
pub trait AiProvider: Send + Sync {
    /// 获取提供商类型
    fn provider_type(&self) -> AiProviderType;

    /// 检查模型可用性
    async fn is_model_available(&self, model: &str) -> bool;

    /// 获取可用模型列表
    async fn list_models(&self) -> AiResult<Vec<ModelInfo>>;

    /// 发送AI请求
    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse>;

    /// 获取配置参数
    fn get_config_keys(&self) -> Vec<&'static str> {
        vec![]
    }

    /// 健康检查
    async fn health_check(&self) -> AiResult<bool> {
        self.list_models().await.map(|_| true)
    }

    /// 计算预估成本
    fn estimate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> Option<f64>;

    /// 检查token限制
    fn check_token_limit(&self, model: &str, max_tokens: usize) -> bool;
}
