use async_trait::async_trait;
use log::debug;
use orion_error::{ErrorOwe, ErrorWith};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::error::AiResult;
use crate::provider::*;
use crate::response_converter::convert_response_from_text;

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiResponse {
    pub choices: Vec<Choice>,
    pub usage: Option<Usage>,
    pub model: String,
}

#[derive(Debug, Deserialize)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    pub total_tokens: usize,
}

#[derive(Debug, Deserialize)]
pub struct Choice {
    pub message: Message,
    pub finish_reason: Option<String>,
    pub tool_calls: Option<Vec<OpenAiToolCall>>,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiToolCall {
    pub index: Option<u32>,
    pub id: String,
    pub r#type: String,
    pub function: OpenAiFunctionCall,
}

#[derive(Debug, Deserialize)]
pub struct OpenAiFunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiTool {
    pub r#type: String,
    pub function: OpenAiFunction,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OpenAiFunction {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiRequestWithTools {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stream: bool,
    tools: Option<Vec<OpenAiTool>>,
    tool_choice: Option<serde_json::Value>,
}

impl OpenAiProvider {
    pub fn convert_to_openai_tools(
        functions: &[crate::provider::FunctionDefinition],
    ) -> Vec<OpenAiTool> {
        functions
            .iter()
            .map(|f| {
                let properties: serde_json::Map<String, serde_json::Value> = f
                    .parameters
                    .iter()
                    .map(|p| {
                        (
                            p.name.clone(),
                            serde_json::json!({
                                "type": Self::map_parameter_type(&p.r#type),
                                "description": p.description
                            }),
                        )
                    })
                    .collect();

                let required: Vec<String> = f
                    .parameters
                    .iter()
                    .filter(|p| p.required)
                    .map(|p| p.name.clone())
                    .collect();

                OpenAiTool {
                    r#type: "function".to_string(),
                    function: OpenAiFunction {
                        name: f.name.clone(),
                        description: f.description.clone(),
                        parameters: serde_json::json!({
                            "type": "object",
                            "properties": properties,
                            "required": required
                        }),
                    },
                }
            })
            .collect()
    }

    fn map_parameter_type(param_type: &str) -> String {
        match param_type {
            "string" => "string".to_string(),
            "array" => "array".to_string(),
            "number" | "integer" => "number".to_string(),
            "boolean" => "boolean".to_string(),
            "object" => "object".to_string(),
            _ => "string".to_string(), // 默认为 string
        }
    }
}

pub struct OpenAiProvider {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
    organization: Option<String>,
    provider_type: AiProviderType,
}

impl OpenAiProvider {
    /// 创建标准的OpenAI Provider
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            organization: None,
            provider_type: AiProviderType::OpenAi,
        }
    }

    /// 创建DeepSeek兼容Provider (100% OpenAI格式兼容)
    pub fn deep_seek(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            api_key,
            base_url: "https://api.deepseek.com/v1".to_string(),
            organization: None,
            provider_type: AiProviderType::DeepSeek,
        }
    }
    pub fn kimi_k2(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            api_key,
            base_url: "https://api.moonshot.cn/v1".to_string(),
            organization: None,
            provider_type: AiProviderType::Kimi,
        }
    }

    /// 创建Groq兼容Provider (OpenAI格式)
    pub fn groq(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            api_key,
            base_url: "https://api.groq.com/openai/v1".to_string(),
            organization: None,
            provider_type: AiProviderType::Groq,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn with_organization(mut self, org: String) -> Self {
        self.organization = Some(org);
        self
    }

    fn create_headers(&self) -> header::HeaderMap {
        let mut headers = header::HeaderMap::new();

        headers.insert(
            header::AUTHORIZATION,
            header::HeaderValue::from_str(&format!("Bearer {}", self.api_key)).unwrap(),
        );

        headers.insert(
            header::CONTENT_TYPE,
            header::HeaderValue::from_static("application/json"),
        );

        if let Some(org) = &self.organization {
            headers.insert(
                reqwest::header::HeaderName::from_static("OpenAI-Organization"),
                header::HeaderValue::from_str(org).unwrap(),
            );
        }

        headers
    }

    fn map_model_to_info(&self, model: &str) -> ModelInfo {
        let model_map: HashMap<&str, (usize, bool, bool, f64, f64, AiProviderType)> =
            HashMap::from([
                (
                    "glm-4.5",
                    (128000, true, false, 0.00007, 0.00028, AiProviderType::Glm),
                ),
                // OpenAI models
                (
                    "gpt-4o",
                    (128000, true, true, 0.005, 0.015, AiProviderType::OpenAi),
                ),
                // DeepSeek models (99.5% cost reduction)
                (
                    "deepseek-chat",
                    (
                        32768,
                        true,
                        false,
                        0.00007,
                        0.00028,
                        AiProviderType::DeepSeek,
                    ),
                ),
                (
                    "deepseek-coder",
                    (
                        32768,
                        true,
                        false,
                        0.00007,
                        0.00028,
                        AiProviderType::DeepSeek,
                    ),
                ),
                (
                    "deepseek-reasoner",
                    (
                        32768,
                        true,
                        true,
                        0.00014,
                        0.00056,
                        AiProviderType::DeepSeek,
                    ),
                ),
                // Groq models
                (
                    "mixtral-8x7b-32768",
                    (32768, false, false, 0.00027, 0.00027, AiProviderType::Groq),
                ),
                (
                    "llama3-70b-8192",
                    (8192, false, false, 0.00059, 0.00079, AiProviderType::Groq),
                ),
                (
                    "gemma2-9b-it",
                    (8192, false, false, 0.00010, 0.00010, AiProviderType::Groq),
                ),
            ]);

        let default = (4096, false, false, 0.001, 0.002, AiProviderType::OpenAi);
        let (
            max_tokens,
            supports_images,
            supports_reasoning,
            input_cost,
            output_cost,
            provider_type,
        ) = model_map.get(model).unwrap_or(&default);

        ModelInfo {
            name: model.to_string(),
            provider: *provider_type,
            max_tokens: *max_tokens,
            supports_images: *supports_images,
            supports_reasoning: *supports_reasoning,
            cost_per_1k_input: *input_cost,
            cost_per_1k_output: *output_cost,
        }
    }
}

#[async_trait]
impl AiProvider for OpenAiProvider {
    fn provider_type(&self) -> AiProviderType {
        self.provider_type
    }

    async fn is_model_available(&self, model: &str) -> bool {
        match self.list_models().await {
            Ok(models) => models.iter().any(|m| m.name == model),
            Err(_) => false,
        }
    }

    async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        let models = match self.provider_type {
            AiProviderType::Glm => vec!["glm-4.5"],
            AiProviderType::OpenAi => vec!["gpt-4o", "gpt-4o-mini", "gpt-4-turbo", "gpt-3.5-turbo"],
            AiProviderType::DeepSeek => {
                vec!["deepseek-chat", "deepseek-coder", "deepseek-reasoner"]
            }
            AiProviderType::Groq => vec!["mixtral-8x7b-32768", "llama3-70b-8192", "gemma2-9b-it"],
            _ => vec!["gpt-4o-mini"],
        };

        Ok(models.iter().map(|m| self.map_model_to_info(m)).collect())
    }

    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse> {
        let system_msg = Message {
            role: "system".to_string(),
            content: request.system_prompt.clone(),
        };

        let user_msg = Message {
            role: "user".to_string(),
            content: request.user_prompt.clone(),
        };

        let openai_request = OpenAiRequest {
            model: request.model.clone(),
            messages: vec![system_msg, user_msg],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
        };
        debug!("send client request: {openai_request:#?}");

        let url = format!("{}/chat/completions", self.base_url);
        debug!("send client url: {url}");
        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&openai_request)
            .send()
            .await
            .owe_res()
            .with(url)?;

        debug!("Client response: {response:#?}");
        println!("{} think....", request.model);

        // Get raw response text first
        let response_text = response.text().await.owe_data()?;
        debug!("Raw response body: {response_text}");

        // 使用高级转换函数（自动解析JSON和转换响应）
        convert_response_from_text(
            &response_text,
            self.provider_type,
            &request.model,
            |model, input_tokens, output_tokens| {
                self.estimate_cost(model, input_tokens, output_tokens)
            },
        )
    }

    fn estimate_cost(&self, model: &str, input_tokens: usize, output_tokens: usize) -> Option<f64> {
        let model_info = self.map_model_to_info(model);
        let cost = (input_tokens as f64 * model_info.cost_per_1k_input / 1000.0)
            + (output_tokens as f64 * model_info.cost_per_1k_output / 1000.0);
        Some(cost)
    }

    fn check_token_limit(&self, model: &str, max_tokens: usize) -> bool {
        let model_info = self.map_model_to_info(model);
        max_tokens <= model_info.max_tokens
    }

    fn get_config_keys(&self) -> Vec<&'static str> {
        match self.provider_type {
            AiProviderType::OpenAi => vec!["OPENAI_API_KEY", "OPENAI_ORG_ID", "OPENAI_BASE_URL"],
            AiProviderType::DeepSeek => vec!["DEEPSEEK_API_KEY", "DEEPSEEK_BASE_URL"],
            AiProviderType::Groq => vec!["GROQ_API_KEY", "GROQ_BASE_URL"],
            _ => vec!["API_KEY", "BASE_URL"],
        }
    }

    fn supports_function_calling(&self) -> bool {
        true // OpenAI 支持函数调用
    }

    async fn send_request_with_functions(
        &self,
        request: &crate::provider::AiRequest,
        functions: &[crate::provider::FunctionDefinition],
    ) -> AiResult<crate::provider::AiResponse> {
        let openai_tools = Self::convert_to_openai_tools(functions);

        let openai_request = OpenAiRequestWithTools {
            model: request.model.clone(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: request.system_prompt.clone(),
                },
                Message {
                    role: "user".to_string(),
                    content: request.user_prompt.clone(),
                },
            ],
            max_tokens: request.max_tokens,
            temperature: request.temperature,
            stream: false,
            tools: Some(openai_tools),
            tool_choice: Some(serde_json::json!("auto")),
        };
        debug!(
            "send client request: {:#}",
            serde_json::to_string(&openai_request).unwrap()
        );

        let url = format!("{}/chat/completions", self.base_url);
        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&openai_request)
            .send()
            .await
            .owe_res()
            .with(url.clone())?;

        let response_text = response.text().await.owe_data()?;
        debug!("Raw response body: {response_text}");

        // 使用高级转换函数（自动解析JSON和转换响应）
        convert_response_from_text(
            &response_text,
            self.provider_type,
            &request.model,
            |model, input_tokens, output_tokens| {
                self.estimate_cost(model, input_tokens, output_tokens)
            },
        )
    }
}
