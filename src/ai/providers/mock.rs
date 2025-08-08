use async_trait::async_trait;
use std::collections::HashMap;
// Removed unused import

use crate::ai::{error::AiResult, provider::*};

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl AiProvider for MockProvider {
    fn provider_type(&self) -> AiProviderType {
        AiProviderType::Mock
    }

    async fn is_model_available(&self, model: &str) -> bool {
        ["mock-gpt", "mock-claude", "mock-local"].contains(&model)
    }

    async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
        Ok(vec![
            ModelInfo {
                name: "mock-gpt".to_string(),
                provider: AiProviderType::Mock,
                max_tokens: 4000,
                supports_images: false,
                supports_reasoning: false,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
            },
            ModelInfo {
                name: "mock-claude".to_string(),
                provider: AiProviderType::Mock,
                max_tokens: 4000,
                supports_images: false,
                supports_reasoning: true,
                cost_per_1k_input: 0.0,
                cost_per_1k_output: 0.0,
            },
        ])
    }

    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse> {
        let content = format!(
            "[MOCK] Response for model: {} with prompt: {:.50}...",
            request.model, request.user_prompt
        );

        Ok(AiResponse {
            content,
            model: request.model.clone(),
            usage: UsageInfo {
                prompt_tokens: request.user_prompt.len() / 4,
                completion_tokens: 50,
                total_tokens: (request.user_prompt.len() / 4) + 50,
                estimated_cost: Some(0.0),
            },
            finish_reason: Some("stop".to_string()),
            provider: AiProviderType::Mock,
            metadata: HashMap::new(),
        })
    }

    fn estimate_cost(
        &self,
        _model: &str,
        _input_tokens: usize,
        _output_tokens: usize,
    ) -> Option<f64> {
        Some(0.0)
    }

    fn check_token_limit(&self, _model: &str, max_tokens: usize) -> bool {
        max_tokens <= 4000
    }

    fn get_config_keys(&self) -> Vec<&'static str> {
        vec!["MOCK_API_KEY"]
    }
}
