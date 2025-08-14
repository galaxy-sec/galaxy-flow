use async_trait::async_trait;
use orion_error::{ErrorOwe, UvsConfFrom};
use reqwest::{header, Client};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ai::error::{AiErrReason, AiResult};
use crate::ai::provider::*;

#[derive(Debug, Serialize, Deserialize)]
struct OpenAiRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    stream: bool,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    choices: Vec<Choice>,
    usage: Option<Usage>,
    model: String,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    prompt_tokens: usize,
    completion_tokens: usize,
    total_tokens: usize,
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

        let url = format!("{}/chat/completions", self.base_url);

        let response = self
            .client
            .post(&url)
            .headers(self.create_headers())
            .json(&openai_request)
            .send()
            .await
            .owe_res()?;

        let response_body = response.json::<OpenAiResponse>().await.owe_data()?;

        let choice = response_body
            .choices
            .first()
            .ok_or_else(|| AiErrReason::from_conf("No choices in response".to_string()))?;

        Ok(AiResponse {
            content: choice.message.content.clone(),
            model: response_body.model.clone(),
            usage: UsageInfo {
                prompt_tokens: response_body
                    .usage
                    .as_ref()
                    .map(|u| u.prompt_tokens)
                    .unwrap_or(0),
                completion_tokens: response_body
                    .usage
                    .as_ref()
                    .map(|u| u.completion_tokens)
                    .unwrap_or(0),
                total_tokens: response_body
                    .usage
                    .as_ref()
                    .map(|u| u.total_tokens)
                    .unwrap_or(0),
                estimated_cost: self.estimate_cost(
                    &request.model,
                    response_body
                        .usage
                        .as_ref()
                        .map(|u| u.prompt_tokens)
                        .unwrap_or(0),
                    response_body
                        .usage
                        .as_ref()
                        .map(|u| u.completion_tokens)
                        .unwrap_or(0),
                ),
            },
            finish_reason: choice.finish_reason.clone(),
            provider: self.provider_type,
            metadata: HashMap::new(),
        })
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
}
