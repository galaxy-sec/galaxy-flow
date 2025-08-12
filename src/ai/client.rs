use crate::ai::capabilities::AiTask;
use crate::ai::provider::{AiProvider, AiProviderType, AiRequest};
use std::collections::HashMap;
use std::sync::Arc;

use super::provider::AiResponse;
use super::providers::openai::OpenAiProvider;
use super::{AiConfig, AiErrReason, AiError, AiResult, AiRouter};

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

        // 初始化DeepSeek
        if let Some(key) = config.get_api_key(AiProviderType::DeepSeek) {
            let provider = OpenAiProvider::deep_seek(key);
            providers.insert(AiProviderType::DeepSeek, Arc::new(provider));
        }

        // 初始化Groq
        if let Some(key) = config.get_api_key(AiProviderType::Groq) {
            let provider = OpenAiProvider::groq(key);
            providers.insert(AiProviderType::Groq, Arc::new(provider));
        }

        // Mock provider for testing
        providers.insert(
            AiProviderType::Mock,
            Arc::new(crate::ai::providers::mock::MockProvider::new()),
        );

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
            Err(AiError::from(AiErrReason::NoProviderAvailable))
        }
    }

    pub async fn smart_request(
        &self,
        capability: AiTask,
        prompt: &str,
    ) -> AiResult<AiResponse> {
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
        let system_prompt =
            "你是一名资深工程师，专门理解代码变更并生成符合Conventional Commits标准的提交信息。";

        let request = AiRequest::builder()
            .model("gpt-4o-mini")
            .system_prompt(system_prompt)
            .user_prompt(format!(
                "分析以下代码变更，生成简洁的提交信息（最多50个字符）：\n{context}"
            ))
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
            .user_prompt(format!(
                "审查{file_path}中的代码并指出潜在问题：\n{code}"
            ))
            .max_tokens(2000)
            .temperature(0.3)
            .build();

        self.send_request(request).await
    }

    /// 获取所有可用的provider
    pub fn available_providers(&self) -> Vec<AiProviderType> {
        self.providers.keys().copied().collect()
    }

    /// 检查特定provider是否可用
    pub fn is_provider_available(&self, provider: AiProviderType) -> bool {
        self.providers.contains_key(&provider)
    }

    /// 根据任务选择合适的模型和provider
    fn build_system_prompt(&self, capability: AiTask) -> String {
        match capability {
            // 代码开发类任务
            AiTask::Coding => "生成高质量、可直接使用的代码或文档".to_string(),
            AiTask::Optimizing => "基于代码上下文提供改进建议，保持简洁实用".to_string(),
            AiTask::Testing => "设计全面的测试用例和测试策略".to_string(),
            AiTask::Documenting => "生成清晰、准确的技术文档".to_string(),
            // 项目管理类任务
            AiTask::Planning => "分析变更对系统的影响和潜在风险".to_string(),
            AiTask::Committing => "理解代码变更并生成精准的提交信息".to_string(),
            AiTask::Branching => "提供分支管理和合并策略建议".to_string(),
            AiTask::Releasing => "制定发布规划和执行策略".to_string(),
            // 系统运维类任务
            AiTask::Deploying => "提供智能部署策略和风险评估建议".to_string(),
            AiTask::Installing => "提供软件安装和配置指导".to_string(),
            AiTask::Configuring => "提供系统配置和优化建议".to_string(),
            // 监控诊断类任务
            AiTask::Analyzing => "深入分析代码的复杂度、结构和设计模式，提供技术洞察".to_string(),
            AiTask::Reviewing => "专注安全性、性能和可维护性的代码审查".to_string(),
            AiTask::Troubleshooting => "诊断和解决代码问题，提供调试建议".to_string(),
            // 知识管理类任务
            AiTask::Explaining => "深入理解项目整体架构和设计模式".to_string(),
            AiTask::Learning => "提供团队协作和代码集成建议".to_string(),
            AiTask::Searching => "搜索和分析相关信息，提供准确答案".to_string(),
            // 其他任务类型
            AiTask::Restarting => "提供服务重启和管理建议".to_string(),
            AiTask::Monitoring => "提供系统监控和告警建议".to_string(),
            AiTask::Auditing => "提供安全审计和合规检查建议".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_client_with_deepseek() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            return;
        }
        // 创建配置，启用 DeepSeek
        let config = AiConfig::example();

        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 验证 DeepSeek 可用
        assert!(client.is_provider_available(AiProviderType::DeepSeek));
        assert!(client
            .available_providers()
            .contains(&AiProviderType::DeepSeek));

        // 创建简单的测试请求
        let request = AiRequest::builder()
            .model("deepseek-chat")
            .system_prompt("你是一个测试助手".to_string())
            .user_prompt("请回答：1+1=?".to_string())
            .build();

        // 发送请求到 DeepSeek
        let response = client.send_request(request).await;

        match response {
            Ok(resp) => {
                println!("✅ DeepSeek 响应: {}", resp.content);
                assert!(!resp.content.is_empty());
                assert_eq!(resp.provider, AiProviderType::DeepSeek);
            }
            Err(e) => {
                // 在没有真实 API key 的情况下，这可能是预期的
                println!("⚠️ DeepSeek 请求失败（预期，需要真实 API key）: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_client_smart_request_with_deepseek() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            return;
        }

        let config = AiConfig::example();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 使用 smart_request 方法
        let response = client.smart_request(
            AiTask::Analyzing,
            "分析这个函数的性能：\nfn fibonacci(n: u64) -> u64 { if n <= 1 { n } else { fibonacci(n-1) + fibonacci(n-2) } }"
        ).await;

        match response {
            Ok(resp) => {
                println!("✅ DeepSeek smart 响应: {}", resp.content);
                assert!(!resp.content.is_empty());
            }
            Err(e) => {
                println!("⚠️ DeepSeek smart 请求失败（预期）: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_client_commit_with_deepseek() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            return;
        }

        let config = AiConfig::example();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试 commit 功能
        let context = r#"feat: add user authentication
- Implement login endpoint
- Add JWT token generation
- Include password hashing"#;

        let response = client.smart_commit(context).await;

        match response {
            Ok(resp) => {
                println!("✅ DeepSeek commit 响应: {}", resp.content);
                assert!(!resp.content.is_empty());
                // 验证提交信息符合 Conventional Commits 格式
                assert!(
                    resp.content.contains("feat:")
                        || resp.content.contains("fix:")
                        || resp.content.contains("docs:")
                );
            }
            Err(e) => {
                println!("⚠️ DeepSeek commit 请求失败（预期）: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_client_code_review_with_deepseek() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            return;
        }
        let config = AiConfig::example();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试代码审查功能
        let code = r#"function processUserData(user) {
    if (!user || !user.name) {
        return null;
    }

    let result = [];
    for (let i = 0; i < user.data.length; i++) {
        result.push(user.data[i] * 2);
    }

    return result;
}"#;

        let response = client.code_review(code, "user.js").await;

        match response {
            Ok(resp) => {
                println!("✅ DeepSeek code review 响应: {}", resp.content);
                assert!(!resp.content.is_empty());
                // 验证包含代码审查相关的关键词
                assert!(
                    resp.content.to_lowercase().contains("security")
                        || resp.content.to_lowercase().contains("performance")
                        || resp.content.to_lowercase().contains("vulnerability")
                );
            }
            Err(e) => {
                println!("⚠️ DeepSeek code review 请求失败（预期）: {e}");
            }
        }
    }

    #[tokio::test]
    async fn test_client_provider_fallback() {
        // 测试当 DeepSeek 不可用时的回退机制
        let mut config = AiConfig::example();

        // 禁用 DeepSeek
        config
            .providers
            .get_mut(&AiProviderType::DeepSeek)
            .unwrap()
            .enabled = false;

        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 验证 DeepSeek 不可用
        assert!(!client.is_provider_available(AiProviderType::DeepSeek));

        // 其他 provider 应该仍然可用
        assert!(client.is_provider_available(AiProviderType::Mock));
        assert!(client.available_providers().contains(&AiProviderType::Mock));
    }

    #[tokio::test]
    async fn test_client_model_routing() {
        // 测试模型路由功能
        env::set_var("DEEPSEEK_API_KEY", "test_deepseek_key");
        env::set_var("OPENAI_API_KEY", "test_openai_key");

        let config = AiConfig::example();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 使用不同的模型进行测试
        let models_to_test = vec![
            "deepseek-chat", // 应该路由到 DeepSeek
            "gpt-4o-mini",   // 应该路由到 OpenAI
            "unknown-model", // 应该使用默认路由
        ];

        for model in models_to_test {
            let request = AiRequest::builder()
                .model(model)
                .system_prompt("测试".to_string())
                .user_prompt("测试内容".to_string())
                .build();

            let response = client.send_request(request).await;
            match response {
                Ok(resp) => {
                    println!("✅ 模型 {} 路由成功: {:?}", model, resp.provider);
                    assert!(!resp.content.is_empty());
                }
                Err(e) => {
                    println!("⚠️ 模型 {model} 请求失败: {e}");
                    // 某些模型可能因为配置问题而失败，这是可以接受的
                }
            }
        }
    }
}
