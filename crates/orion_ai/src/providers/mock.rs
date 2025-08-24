use async_trait::async_trait;
use std::collections::HashMap;
// Removed unused import

use crate::{error::AiResult, provider::*};

pub struct MockProvider;

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
    }
}

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
        // MockProvider 支持所有模型名称，这样就能测试 function calling
        true
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
            ModelInfo {
                name: "mock".to_string(),
                provider: AiProviderType::Mock,
                max_tokens: 4000,
                supports_images: false,
                supports_reasoning: false,
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
            function_calls: None,
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

    async fn send_request_with_functions(
        &self,
        request: &AiRequest,
        _functions: &[FunctionDefinition],
    ) -> AiResult<AiResponse> {
        // 模拟函数调用 - 根据用户提示决定是否调用函数
        let function_calls = if request.user_prompt.contains("git_status") {
            Some(vec![FunctionCall {
                name: "git_status".to_string(),
                arguments: std::collections::HashMap::from([(
                    "path".to_string(),
                    serde_json::Value::String(".".to_string()),
                )]),
            }])
        } else if request.user_prompt.contains("git_add") {
            Some(vec![FunctionCall {
                name: "git_add".to_string(),
                arguments: std::collections::HashMap::from([(
                    "files".to_string(),
                    serde_json::Value::Array(vec![serde_json::Value::String(".".to_string())]),
                )]),
            }])
        } else if request.user_prompt.contains("git_commit") {
            Some(vec![FunctionCall {
                name: "git_commit".to_string(),
                arguments: std::collections::HashMap::from([(
                    "message".to_string(),
                    serde_json::Value::String("Mock commit message".to_string()),
                )]),
            }])
        } else if request.user_prompt.contains("git_push") {
            Some(vec![FunctionCall {
                name: "git_push".to_string(),
                arguments: std::collections::HashMap::from([
                    (
                        "remote".to_string(),
                        serde_json::Value::String("origin".to_string()),
                    ),
                    (
                        "branch".to_string(),
                        serde_json::Value::String("main".to_string()),
                    ),
                ]),
            }])
        } else {
            None
        };

        let content = if function_calls.is_some() {
            "[MOCK] I will call the Git functions to help you.".to_string()
        } else {
            format!(
                "[MOCK] Response for model: {} with prompt: {:.50}...",
                request.model, request.user_prompt
            )
        };

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
            function_calls,
        })
    }

    fn supports_function_calling(&self) -> bool {
        true
    }
}
