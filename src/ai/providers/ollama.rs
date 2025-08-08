use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ai::{provider::*, error::AiResult, AiProviderType};

#[derive(Debug, Serialize, Deserialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    options: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
    model: String,
    done: bool,
}

pub struct OllamaProvider {
    client: Arc<Client>,
    base_url: String,
}

impl OllamaProvider {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(60))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            base_url: "http://localhost:11434".to_string(),
        }
    }

    #[cfg(test)]
    pub fn with_base_url(url: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            base_url: url,
        }
    }
}

#[async_trait]
impl AiProvider for OllamaProvider {
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Ollama
    }

    async fn is_model_available(&self, model: &str) -> bool {
        matches!(
            model,
            "deepseek-coder"|"codellama"|"llama3"|"mistral"
        )
    }

    async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        let models = vec![
            ModelInfo {
                name: "deepseek-coder".to_string(),
                provider: AiProviderType::Ollama,
                max_tokens: 16_000,
                supports_images: false,
                supports_reasoning: true,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
            },
            ModelInfo {
                name: "codellama".to_string(),
                provider: AiProviderType::Ollama,
                max_tokens: 32_000,
                supports_images: false,
                supports_reasoning: true,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
            },
            ModelInfo {
                name: "llama3".to_string(),
                provider: AiProviderType::Ollama,
                max_tokens: 8_000,
                supports_images: false,
                supports_reasoning: false,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
            },
        ];

        Ok(models)
    }

    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse> {
        let ollama_request = OllamaRequest {
            model: request.model.clone(),
            prompt: format!("{}{}{}",
                request.system_prompt,
                if !request.system_prompt.is_empty() { "\n\n" } else { "" },
                request.user_prompt
            ),
            stream: false,
            options: None,
        };

        let url = format!("{}/api/generate", self.base_url);

        let response = self.client
            .post(&url)
            .json(&ollama_request)
            .send()
            .await?;

        let ollama_response = response.json::<OllamaResponse>().await?;

        Ok(AiResponse {
            content: ollama_response.response.trim().to_string(),
            model: ollama_response.model,
            usage: UsageInfo {
                prompt_tokens: request.user_prompt.len() / 4,
                completion_tokens: ollama_response.response.len() / 4,
                total_tokens: (request.user_prompt.len() + ollama_response.response.len()) / 4,
                estimated_cost: Some(0.0),
            },
            finish_reason: Some("stop".to_string()),
            provider: AiProviderType::Ollama,
            metadata: HashMap::new(),
        })
    }

    fn estimate_cost(&self, _model: &str, _input_tokens: usize, _output_tokens: usize) -> Option<f64> {
        Some(0.0)
    }

    fn check_token_limit(&self, _model: &str, _max_tokens: usize) -> bool {
        true
    }

    fn get_config_keys(&self) -> Vec<&'static str> {
        vec!["OLLAMA_MODEL
