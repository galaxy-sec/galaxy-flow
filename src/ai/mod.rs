mod provider;
mod error;
mod config;
mod capabilities;

pub use provider::*;
pub use error::*;
pub use config::*;
pub use capabilities::*;
pub use context::*;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use crate::ai::providers::openai::OpenAiProvider;
use crate::ai::providers::anthropic::AnthropicProvider;
use crate::ai::providers::ollama::OllamaProvider;
use crate::ai::providers::mock::MockProvider;
use crate::ai::router::AiRouter;

/// 主AI客户端，统一的API接口
pub struct AiClient {
    providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    config: Arc<AiConfig>,
    router: AiRouter,
}

impl AiClient {
    pub fn new(config: AiConfig) -> AiResult<Self> {
        let mut providers: HashMap<AiProviderType, Arc<dyn AiProvider>> = HashMap::new();

        // 初始化支持的provider
        if let Some(key) = config.get_api_key(AiProviderType::OpenAi) {
            let provider = OpenAiProvider::new(key);
            providers.insert(AiProviderType::OpenAi, Arc::new(provider));
        }

        if let Some(key) = config.get_api_key(AiProviderType::Anthropic) {
            let provider = AnthropicProvider::new(key);
            providers.insert(AiProviderType::Anthropic, Arc::new(provider));
        }

        // Ollama本地模型（总是添加）
        providers.insert(AiProviderType::Ollama, Arc::new(OllamaProvider::new()));

        // Mock provider for testing
        providers.insert(AiProviderType::Mock, Arc::new(MockProvider::new()));

        Ok(Self {
            providers,
            config: Arc::new(config),
            router: AiRouter::new(),
        })
    }

    pub async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let provider_type = self.router.select_provider(&request.model, &self.config);

        if let Some(provider) = self.providers.get(&provider_type) {
            provider.send_request(&request).await
        } else {
            Err(AiError::NoProviderAvailable)
        }
    }

    pub async fn smart_request(&self, capability: AiCapability, prompt: &str) -> AiResult<AiResponse> {
        let model = capability.recommended_model();
        let system_prompt = self.build_system_prompt(capability);

        let request = AiRequest::builder()
            .model(model)
            .system_prompt(system_prompt)
            .user_prompt(prompt)
            .capability(capability)
            .build();

        self.send_request(request).await
    }

    pub async fn smart_commit(&self, context: &str) -> AiResult<AiResponse> {
        let system_prompt = "你是一名资深工程师，专门理解代码变更并生成符合Conventional Commits标准的提交信息。";

        let request = AiRequest::builder()
            .model("gpt-4o-mini")
            .system_prompt(system_prompt)
            .user_prompt(&format!("分析以下代码变更，生成简洁的提交信息（最多50个字符）：\n{}", context))
            .max_tokens(75)
            .temperature(0.7)
            .build();

        self.send_request(request).await
    }

    pub async fn code_review(&self, code: &str, file_path: &str) -> AiResult<AiResponse> {
        let system_prompt = "你是一名代码审查专家，专注于安全性、性能和可维护性。";

        let request = AiRequest::builder()
            .model("claude-3-5-sonnet")
            .system_prompt(system_prompt)
            .user_prompt(&format!("审查{}中的代码并指出潜在问题：\n{}", file_path, code))
            .max_tokens(2000)
            .temperature(0.3)
            .build();

        self.send_request(request).await
    }

    /// 获取所有可用的provider
    pub fn available_providers(&self) -> Vec<AiProviderType> {
+        self.providers.keys().copied().collect()
+    }
+
+    /// 检查特定provider是否可用
+    pub fn is_provider_available(&self, provider: AiProviderType) -> bool {
+        self.providers.contains_key(&provider)
+    }
+
+    /// 根据能力选择合适的模型和provider
+    fn build_system_prompt(&self, capability: AiCapability) -> String {
+        match capability {
+            AiCapability::Analyze => "深入分析代码的复杂度、结构和设计模式，提供技术洞察".to_string(),
+            AiCapability::Suggest => "基于代码上下文提供改进建议，保持简洁实用".to_string(),
+            AiCapability::Check => "检查代码的安全、性能和可维护性问题".to_string(),
+            AiCapability::Generate => "生成高质量、可直接使用的代码或文档".to_string(),
+            AiCapability::Refactor => "提供具体的重构建议，
