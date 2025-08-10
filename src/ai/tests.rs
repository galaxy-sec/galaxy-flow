#[cfg(test)]
mod tests {
    use crate::ai::{
        capabilities::AiDevCapability,
        client::AiClient,
        provider::{AiProvider, AiProviderType, AiRequestBuilder},
        providers::{mock::MockProvider, openai::OpenAiProvider},
        AiConfig, AiError,
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
    async fn test_deepseek_provider_instantiation() -> Result<(), Box<dyn Error>> {
        println!("🔧 Testing DeepSeek Provider Instantiation...");

        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let provider = OpenAiProvider::deep_seek(api_key.clone());

            // 验证Provider类型
            assert_eq!(provider.provider_type(), AiProviderType::DeepSeek);

            // 验证配置键
            let config_keys = provider.get_config_keys();
            assert!(config_keys.contains(&"DEEPSEEK_API_KEY"));
            assert!(config_keys.contains(&"DEEPSEEK_BASE_URL"));

            // 验证模型可用性检查
            assert!(provider.is_model_available("deepseek-chat").await);
            assert!(provider.is_model_available("deepseek-coder").await);
            assert!(!provider.is_model_available("gpt-4o").await); // 非DeepSeek模型

            println!("✅ DeepSeek Provider instantiation successful");
            println!("📝 Config keys: {:?}", config_keys);
        } else {
            println!("⚠️ Skipping DeepSeek provider tests - no DEEPSEEK_API_KEY");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_deepseek_model_list() -> Result<(), Box<dyn Error>> {
        println!("📋 Testing DeepSeek Model List...");

        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let provider = OpenAiProvider::deep_seek(api_key);

            // 验证模型列表
            let models = provider.list_models().await?;
            assert!(!models.is_empty());

            // 验证DeepSeek模型存在
            let model_names: Vec<&str> = models.iter().map(|m| m.name.as_str()).collect();
            assert!(model_names.contains(&"deepseek-chat"));
            assert!(model_names.contains(&"deepseek-coder"));
            assert!(model_names.contains(&"deepseek-reasoner"));

            // 验证模型信息
            for model in models {
                assert_eq!(model.provider, AiProviderType::DeepSeek);
                assert!(model.max_tokens > 0);
                assert!(model.cost_per_1k_input >= 0.0);
                assert!(model.cost_per_1k_output >= 0.0);

                println!(
                    "📊 Model: {}, Max tokens: {}, Cost: ${}/{}/1k tokens",
                    model.name, model.max_tokens, model.cost_per_1k_input, model.cost_per_1k_output
                );
            }

            println!("✅ DeepSeek model list verification successful");
        } else {
            println!("⚠️ Skipping DeepSeek model list tests - no DEEPSEEK_API_KEY");
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
    async fn test_deepseek_basic_api_call() -> Result<(), Box<dyn Error>> {
        println!("🌐 Testing DeepSeek Basic API Call...");

        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            let provider = OpenAiProvider::deep_seek(api_key);

            let request = AiRequestBuilder::new()
                .model("deepseek-chat")
                .system_prompt("你是一个编程助手")
                .user_prompt("用一句话解释什么是零依赖设计")
                .capability(AiDevCapability::Suggest)
                .max_tokens(100)
                .temperature(0.7)
                .build();

            let response = provider.send_request(&request).await?;

            // 验证基本响应
            assert!(!response.content.is_empty());
            assert_eq!(response.provider, AiProviderType::DeepSeek);
            assert!(response.model.contains("deepseek"));

            // 验证使用统计
            assert!(response.usage.prompt_tokens > 0);
            assert!(response.usage.completion_tokens > 0);
            assert!(response.usage.total_tokens > 0);
            assert!(response.usage.estimated_cost.is_some());

            println!("🟢 DeepSeek API call successful");
            println!("💬 Response: {}", response.content.trim());
            println!(
                "📊 Usage: {} input + {} output = {} tokens",
                response.usage.prompt_tokens,
                response.usage.completion_tokens,
                response.usage.total_tokens
            );
            println!(
                "💰 Cost: ${:.6}",
                response.usage.estimated_cost.unwrap_or(0.0)
            );
        } else {
            println!("⚠️ Skipping DeepSeek API call tests - no DEEPSEEK_API_KEY");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_deepseek_error_handling() -> Result<(), Box<dyn Error>> {
        println!("🚨 Testing DeepSeek Error Handling...");

        if let Ok(api_key) = std::env::var("DEEPSEEK_API_KEY") {
            // 测试无效模型错误
            let provider = OpenAiProvider::deep_seek(api_key.clone());

            let invalid_request = AiRequestBuilder::new()
                .model("invalid-model-name")
                .system_prompt("测试")
                .user_prompt("测试")
                .build();

            let result = provider.send_request(&invalid_request).await;
            assert!(result.is_err());

            match result {
                Err(AiError::NetworkError(_))
                | Err(AiError::AuthError(_))
                | Err(AiError::InvalidModel(_)) => {
                    println!("✅ Expected error caught for invalid model");
                }
                Err(other_error) => {
                    panic!(
                        "Expected network/auth/model error, but got: {:?}",
                        other_error
                    );
                }
                Ok(_) => {
                    panic!("Expected error for invalid model, but got success");
                }
            }

            // 测试无效API密钥（使用无效密钥）
            let invalid_provider = OpenAiProvider::deep_seek("invalid-api-key".to_string());
            let valid_request = AiRequestBuilder::new()
                .model("deepseek-chat")
                .system_prompt("测试")
                .user_prompt("测试")
                .build();

            let invalid_result = invalid_provider.send_request(&valid_request).await;
            assert!(invalid_result.is_err());

            println!("✅ DeepSeek error handling verification successful");
        } else {
            println!("⚠️ Skipping DeepSeek error handling tests - no DEEPSEEK_API_KEY");
        }

        Ok(())
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
    // DEEPSEEK_API_KEY=your_key cargo test deepseek -- --nocapture
    // ./test_openai.sh
}
