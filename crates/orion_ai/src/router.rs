use crate::{provider::AiProviderType, AiConfig};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct AiRouter {
    rules: HashMap<String, AiProviderType>,
}

impl AiRouter {
    pub fn new() -> Self {
        let mut rules = HashMap::new();

        // 默认路由规则
        rules.insert("simple".to_string(), AiProviderType::OpenAi);
        rules.insert("complex".to_string(), AiProviderType::OpenAi);
        rules.insert("free".to_string(), AiProviderType::DeepSeek);

        Self { rules }
    }

    pub fn select_provider(&self, model_name: &str, _config: &AiConfig) -> AiProviderType {
        // 简单的路由逻辑：根据模型名前缀选择provider
        if model_name.starts_with("glm") {
            AiProviderType::Glm
        } else if model_name.starts_with("gpt-") {
            AiProviderType::OpenAi
        } else if model_name.starts_with("claude") || model_name.starts_with("anthropic") {
            AiProviderType::Anthropic
        } else if model_name.starts_with("deepseek") {
            AiProviderType::DeepSeek
        } else if model_name.starts_with("mixtral")
            || model_name.starts_with("llama3")
            || model_name.starts_with("gemma")
        {
            AiProviderType::Groq
        } else if model_name.starts_with("codellama") || model_name.starts_with("llama") {
            AiProviderType::Ollama
        } else if model_name == "mock" {
            AiProviderType::Mock
        } else {
            // 默认使用OpenAI
            AiProviderType::OpenAi
        }
    }

    pub fn register_rule(&mut self, category: String, provider: AiProviderType) {
        self.rules.insert(category, provider);
    }
}

impl Default for AiRouter {
    fn default() -> Self {
        Self::new()
    }
}
