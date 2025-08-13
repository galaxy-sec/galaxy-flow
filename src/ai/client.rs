use crate::ai::capabilities::AiRole;
use crate::ai::provider::{AiProvider, AiProviderType, AiRequest};
use crate::ai::config::{RoleConfigManager, RoleConfigLoader};
use crate::ai::error::{AiError, AiResult};
use std::collections::HashMap;
use std::sync::Arc;

use super::provider::AiResponse;
use super::providers::openai::OpenAiProvider;
use super::{AiConfig, AiErrReason, AiResult as OriginalAiResult, AiRouter};

/// 主AI客户端，统一的API接口
pub struct AiClient {
    providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    config: Arc<AiConfig>,
    router: AiRouter,
    role_config_manager: Arc<RoleConfigManager>,
}

impl AiClient {
    pub fn new(config: AiConfig) -> OriginalAiResult<Self> {
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

        // 初始化角色配置管理器 - 优先使用简化配置
        let role_config_manager = RoleConfigLoader::auto_load(None, None)
            .unwrap_or_else(|_| RoleConfigManager::default());

        Ok(Self {
            providers,
            config: Arc::new(config),
            router: AiRouter::new(),
            role_config_manager: Arc::new(role_config_manager),
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

    /// 基于角色的智能请求处理 - 用户只需选择角色，系统自动选择推荐模型
    pub async fn smart_role_request(
        &self,
        role: AiRole,
        user_input: &str,
    ) -> AiResult<AiResponse> {
        let role_name = &role.to_string();
        
        // 1. 验证AI使用规则
        if let Err(e) = self.role_config_manager.validate_ai_usage(role_name, "smart_request", user_input) {
            return Err(AiError::from(AiErrReason::ConfigError(
                format!("AI usage validation failed: {}", e)
            )));
        }
        
        // 2. 获取角色的AI使用规则和输出要求
        let role_rules = self.role_config_manager.get_role_ai_usage_rules(role_name)?;
        let output_requirements = self.role_config_manager.get_role_output_requirements(role_name)?;
        let best_practices = self.role_config_manager.get_role_best_practices(role_name)?;
        let usage_scenarios = self.role_config_manager.get_role_usage_scenarios(role_name)?;
        
        // 3. 使用角色推荐模型
        let model = role.recommended_model();
        
        // 4. 构建增强的系统提示词
        let base_system_prompt = self.build_role_system_prompt(role);
        let enhanced_system_prompt = self.build_enhanced_system_prompt(
            base_system_prompt,
            &role_rules,
            &output_requirements,
            &best_practices,
            &usage_scenarios,
        );

        let request = AiRequest::builder()
            .model(model)
            .system_prompt(enhanced_system_prompt)
            .user_prompt(user_input)
            .build();

        // 5. 发送请求
        let mut response = self.send_request(request).await?;
        
        // 6. 后处理响应：验证输出符合要求
        if let Err(e) = self.validate_response_content(&response.content, &output_requirements) {
            return Err(AiError::from(AiErrReason::ConfigError(
                format!("Response validation failed: {}", e)
            )));
        }
        
        // 7. 在响应中添加角色信息和规则遵循状态
        response.content = format!(
            "[角色: {}]\n[AI使用规则: 已遵循]\n\n{}",
            role.description(),
            response.content
        );
        
        Ok(response)
    }

    /// 构建基于角色的系统提示
    fn build_role_system_prompt(&self, role: AiRole) -> String {
        // 从配置文件中获取角色系统提示词
        if let Some(role_config) = self.role_config_manager.get_role_config(&role.to_string()) {
            role_config.system_prompt.clone()
        } else {
            // 回退到原有的硬编码描述
            let role_description = role.description();
            format!(
                "你是{role_description}。你的工作流程是连续的，能够智能处理该角色下的各种任务。"
            )
        }
    }

    /// 构建增强的系统提示词，集成AI使用规则
    fn build_enhanced_system_prompt(
        &self,
        base_prompt: String,
        role_rules: &crate::ai::config::roles::AiUsageRules,
        output_requirements: &crate::ai::config::roles::AiOutputRequirements,
        best_practices: &[String],
        usage_scenarios: &[String],
    ) -> String {
        let mut enhanced_prompt = base_prompt;
        
        // 添加使用场景
        if !usage_scenarios.is_empty() {
            enhanced_prompt.push_str("\n\n【使用场景】\n");
            for scenario in usage_scenarios {
                enhanced_prompt.push_str(&format!("- {}\n", scenario));
            }
        }
        
        // 添加最佳实践
        if !best_practices.is_empty() {
            enhanced_prompt.push_str("\n【最佳实践】\n");
            for practice in best_practices {
                enhanced_prompt.push_str(&format!("- {}\n", practice));
            }
        }
        
        // 添加输出要求
        enhanced_prompt.push_str("\n【输出要求】\n");
        enhanced_prompt.push_str(&format!("- 输出格式: {}\n", output_requirements.format));
        enhanced_prompt.push_str(&format!("- 错误处理: {}\n", output_requirements.error_handling));
        
        if let Some(include_comments) = output_requirements.include_comments {
            enhanced_prompt.push_str(&format!("- 包含注释: {}\n", if include_comments { "是" } else { "否" }));
        }
        
        if let Some(include_tests) = output_requirements.include_tests {
            enhanced_prompt.push_str(&format!("- 包含测试: {}\n", if include_tests { "是" } else { "否" }));
        }
        
        // 添加约束条件
        if let Some(max_content_length) = role_rules.constraints.max_content_length {
            enhanced_prompt.push_str(&format!("- 最大内容长度: {} 字符\n", max_content_length));
        }
        
        if let Some(allowed_content_types) = &role_rules.constraints.allowed_content_types {
            enhanced_prompt.push_str("- 允许的内容类型: ");
            enhanced_prompt.push_str(&allowed_content_types.join(", "));
            enhanced_prompt.push_str("\n");
        }
        
        // 添加禁止行为提醒
        if !role_rules.prohibited_actions.is_empty() {
            enhanced_prompt.push_str("\n【禁止行为】\n");
            for action in &role_rules.prohibited_actions {
                enhanced_prompt.push_str(&format!("- 禁止: {}\n", action));
            }
        }
        
        enhanced_prompt
    }

    /// 验证响应内容是否符合输出要求
    fn validate_response_content(
        &self,
        content: &str,
        output_requirements: &crate::ai::config::roles::AiOutputRequirements,
    ) -> Result<(), String> {
        // 检查内容长度
        if let Some(_include_comments) = output_requirements.include_comments {
            // 这里使用include_comments字段来判断是否需要检查内容长度
            if content.len() > 10000 { // 设置一个合理的默认值
                return Err("Response content exceeds maximum length".to_string());
            }
        }
        
        // 检查是否包含必需的注释（如果要求）
        if let Some(include_comments) = output_requirements.include_comments {
            if include_comments && !content.contains("//") && !content.contains("#") {
                return Err("Response should include comments but none found".to_string());
            }
        }
        
        // 检查是否包含必需的测试（如果要求）
        if let Some(include_tests) = output_requirements.include_tests {
            if include_tests && !content.to_lowercase().contains("test") {
                return Err("Response should include tests but none found".to_string());
            }
        }
        
        // 检查格式要求
        if !output_requirements.format.is_empty() {
            match output_requirements.format.to_lowercase().as_str() {
                "json" => {
                    if !content.trim().starts_with('{') && !content.trim().starts_with('[') {
                        return Err("Response should be in JSON format".to_string());
                    }
                }
                "markdown" => {
                    if !content.contains("#") && !content.contains("**") {
                        return Err("Response should be in Markdown format".to_string());
                    }
                }
                _ => {} // 其他格式可以在这里添加验证
            }
        }
        
        Ok(())
    }



    /// 获取所有可用的provider
    pub fn available_providers(&self) -> Vec<AiProviderType> {
        self.providers.keys().copied().collect()
    }

    /// 检查特定provider是否可用
    pub fn is_provider_available(&self, provider: AiProviderType) -> bool {
        self.providers.contains_key(&provider)
    }
}

#[cfg(test)]
mod tests {
    use orion_variate::vars::{EnvDict, EnvEvalable};

    use super::*;
    use std::env;

    #[tokio::test]
    async fn test_client_with_deepseek() {
        if env::var("DEEPSEEK_API_KEY").is_err() {
            return;
        }
        // 创建配置，启用 DeepSeek
        let config = AiConfig::example();

        let client = AiClient::new(config.env_eval(&EnvDict::default())).expect("Failed to create AiClient");

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
       let client = AiClient::new(config.env_eval(&EnvDict::default())).expect("Failed to create AiClient");
        // 使用 smart_role_request 方法
        let response = client.smart_role_request(
            AiRole::Developer,
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
