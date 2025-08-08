#[cfg(test)]
mod tests {
    use crate::ai::{
        capabilities::AiDevCapability,
        client::AiClient,
        provider::{AiProvider, AiRequestBuilder},
        providers::{mock::MockProvider, openai::OpenAiProvider},
        AiConfig,
    };

    use std::error::Error;

    // 🎯 实际可用的测试用例

    #[tokio::test]
    async fn test_mock_provider_workflow() -> Result<(), Box<dyn Error>> {
        println!("🔧 Testing Mock Provider...");

        let provider = MockProvider::new();

        let request = AiRequestBuilder::new()
            .model("mock-model")
            .system_prompt("你是一个Rust工程师")
            .user_prompt("用一句话解释所有权机制")
            .capability(AiDevCapability::Explain)
            .max_tokens(50)
            .build();

        let response = provider.send_request(&request).await?;

        assert!(!response.content.is_empty());
        println!("✅ Mock Response: {}", response.content);
        Ok(())
    }

    #[tokio::test]
    async fn test_openai_integration_if_available() -> Result<(), Box<dyn Error>> {
        println!("🔍 Testing OpenAI Integration...");

        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let provider = OpenAiProvider::new(api_key);

            let request = AiRequestBuilder::new()
                .model("gpt-4o-mini")
                .system_prompt("你是一个编程助手")
                .user_prompt("用一句话解释什么是零依赖")
                .capability(AiDevCapability::Suggest)
                .max_tokens(75)
                .temperature(0.7)
                .build();

            let response = provider.send_request(&request).await?;

            assert!(!response.content.is_empty());
            println!("🟢 OpenAI Response: {}", response.content);
        } else {
            println!("⚠️ Skipping OpenAI - no API key");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_ai_client_e2e() -> Result<(), Box<dyn Error>> {
        println!("🎯 Testing AI Client End-to-End...");

        let config = AiConfig::load()?;
        let client = AiClient::new(config)?;

        // Test 1: Basic request
        let response = client
            .smart_request(
                AiDevCapability::Generate,
                "写一个简单的Hello World Rust程序",
            )
            .await?;

        assert!(!response.content.is_empty());
        println!("🐳 Generated: {}", response.content.trim());

        // Test 2: Smart commit
        let commit = client
            .smart_commit("修复了输入验证逻辑，添加了边界检查")
            .await?;

        assert!(!commit.content.is_empty());
        assert!(commit.content.len() <= 75);
        println!("📝 Smart commit: {}", commit.content);

        // Test 3: Code review
        let review = client
            .code_review("fn divide(a:i32, b:i32) -> i32 { a / b }", "math_utils.rs")
            .await?;

        assert!(!review.content.is_empty());
        println!("🔍 Code review: {}", review.content);

        Ok(())
    }

    #[test]
    fn test_module_compilation() {
        // Basic compilation test
        assert!(true);
    }

    #[test]
    fn test_capability_models() {
        assert_eq!(AiDevCapability::Analyze.recommended_model(), "gpt-4o-mini");
        assert_eq!(AiDevCapability::Generate.recommended_model(), "gpt-4o");
        assert_eq!(
            AiDevCapability::Review.recommended_model(),
            "claude-3-5-sonnet"
        );
        assert_eq!(AiDevCapability::Commit.recommended_model(), "gpt-4o-mini");
    }

    #[tokio::test]
    async fn test_zero_config_mode() -> Result<(), Box<dyn Error>> {
        // Test the core functionality works even in "zero-config" mode
        let config = AiConfig::default();
        let client = AiClient::new(config)?;

        let providers = client.available_providers();
        assert!(!providers.is_empty());

        println!("✅ Available providers: {:?}", providers);
        Ok(())
    }

    // 🚀 实际运行命令：
    // cargo test --test ai_tests -- --nocapture
    // OPENAI_API_KEY=your_key cargo test -- --test-threads=1
    // ./test_openai.sh
}
