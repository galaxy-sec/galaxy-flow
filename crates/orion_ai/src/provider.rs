use crate::error::AiErrReason;
use async_trait::async_trait;
use orion_error::ToStructError;

use orion_error::UvsLogicFrom;
use serde::{Deserialize, Serialize};

use crate::AiResult;

use super::roleid::AiRoleID;

/// AI提供商类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum AiProviderType {
    OpenAi,
    Anthropic,
    Ollama,
    Mock,
    DeepSeek,
    Groq,
    Kimi,
    Glm,
}

impl std::fmt::Display for AiProviderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AiProviderType::OpenAi => write!(f, "openai"),
            AiProviderType::Anthropic => write!(f, "anthropic"),
            AiProviderType::Ollama => write!(f, "ollama"),
            AiProviderType::Mock => write!(f, "mock"),
            AiProviderType::DeepSeek => write!(f, "deepseek"),
            AiProviderType::Groq => write!(f, "groq"),
            AiProviderType::Kimi => write!(f, "kimi"),
            AiProviderType::Glm => write!(f, "glm"),
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
            AiProviderType::DeepSeek => "deepseek",
            AiProviderType::Groq => "groq",
            AiProviderType::Kimi => "kimi",
            AiProviderType::Glm => "glm",
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

/// 统一AI请求结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiRequest {
    pub model: String,
    pub system_prompt: String,
    pub user_prompt: String,
    pub max_tokens: Option<usize>,
    pub temperature: Option<f32>,
    pub role: Option<AiRoleID>,
    // 新增：简单的函数调用支持
    pub functions: Option<Vec<FunctionDefinition>>,
    pub enable_function_calling: bool,
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
    role: Option<AiRoleID>,
    functions: Option<Vec<FunctionDefinition>>,
    enable_function_calling: bool,
}

impl Default for AiRequestBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AiRequestBuilder {
    pub fn new() -> Self {
        Self {
            model: "gpt-4o-mini".to_string(),
            system_prompt: String::new(),
            user_prompt: String::new(),
            max_tokens: None,
            temperature: Some(0.7),
            role: None,
            functions: None,
            enable_function_calling: false,
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

    pub fn role(mut self, role: AiRoleID) -> Self {
        self.role = Some(role);
        self
    }

    pub fn functions(mut self, functions: Vec<FunctionDefinition>) -> Self {
        self.functions = Some(functions);
        self
    }

    pub fn enable_function_calling(mut self, enabled: bool) -> Self {
        self.enable_function_calling = enabled;
        self
    }

    pub fn build(self) -> AiRequest {
        AiRequest {
            model: self.model,
            system_prompt: self.system_prompt,
            user_prompt: self.user_prompt,
            max_tokens: self.max_tokens,
            temperature: self.temperature,
            role: self.role,
            functions: self.functions,
            enable_function_calling: self.enable_function_calling,
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
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    // 新增：函数调用结果
    pub tool_calls: Option<Vec<FunctionCall>>,
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

    /// 检查是否支持 function calling
    fn supports_function_calling(&self) -> bool {
        false
    }

    /// 发送带函数调用的请求 - 简化版本
    async fn send_request_with_functions(
        &self,
        _request: &AiRequest,
        _functions: &[FunctionDefinition],
    ) -> AiResult<AiResponse> {
        Err(AiErrReason::from_logic("TODO: function calling not supported".to_string()).to_err())
    }
}

/// 函数参数 - 简化版本
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionParameter {
    pub name: String,
    pub description: String,
    pub r#type: String, // 直接使用字符串类型描述
    pub required: bool,
}

/// 函数定义 - 简化版本
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<FunctionParameter>,
}

/// 函数调用请求 - 匹配 OpenAI 和 DeepSeek API 格式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCall {
    pub index: Option<u32>,
    pub id: String,
    pub r#type: String,
    pub function: FunctionCallInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionCallInfo {
    pub name: String,
    pub arguments: String,
}

/// 函数调用结果 - 简化版本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    pub name: String,
    pub result: serde_json::Value,
    pub error: Option<String>,
}
