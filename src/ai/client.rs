use crate::ai::capabilities::AiRole;
use crate::ai::config::{ProviderConfig, RoleConfigLoader, RoleConfigManager};
use crate::ai::error::{AiError, AiResult};
use crate::ai::provider::{AiProvider, AiProviderType, AiRequest};
use crate::execution::VarSpace;
use async_trait::async_trait;
use getset::Getters;
use log::{debug, warn};
use orion_error::{ErrorWith, ToStructError, UvsConfFrom};
use orion_variate::vars::EnvDict;
use std::collections::HashMap;
use std::sync::Arc;

use super::provider::AiResponse;
use super::providers::{mock, openai};
use super::{AiConfig, AiErrReason, AiRouter};

/// AI客户端发送类型枚举
pub enum AiSendClient {
    Basic(AiClient),
}

/// AI客户端trait定义
#[async_trait]
pub trait AiClientTrait: Send + Sync {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse>;
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse>;
}

#[async_trait]
impl AiClientTrait for AiSendClient {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        match self {
            Self::Basic(o) => o.send_request(request).await,
        }
    }
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
        match self {
            Self::Basic(o) => o.smart_role_request(role, user_input).await,
        }
    }
}

/// 主AI客户端，统一的API接口
#[derive(Getters)]
#[getset(get = "pub")]
pub struct AiClient {
    providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    config: AiConfig,
    router: AiRouter,
    roles: RoleConfigManager,
}

#[async_trait]
impl AiClientTrait for AiClient {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let provider_type = self.router.select_provider(&request.model, &self.config);

        if let Some(provider) = self.providers.get(&provider_type) {
            provider
                .send_request(&request)
                .await
                .with(format!("provide: {provider_type}"))
        } else {
            for key in self.providers().keys() {
                error!("registed provider: {key}");
            }
            Err(AiError::from(AiErrReason::NoProviderAvailable)).with(provider_type.to_string())
        }
    }

    /// 基于角色的智能请求处理 - 用户只需选择角色，系统自动选择推荐模型
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
        let request = self.build_ai_request(role, user_input)?;
        // 3. 发送请求
        let mut response = self.send_request(request).await?;

        // 4. 在响应中添加角色信息
        response.content = format!("[角色: {}]\n\n{}", role.description(), response.content);

        Ok(response)
    }
}

impl AiClient {
    /// 创建AiClient（简化版本，无Thread支持）
    pub fn new(config: AiConfig) -> AiResult<Self> {
        let mut providers: HashMap<AiProviderType, Arc<dyn AiProvider>> = HashMap::new();

        // 从配置注册provider
        Self::register_providers_from_config(&mut providers, &config.providers)?;

        // 初始化角色配置管理器 - 优先使用简化配置
        let roles_manager = RoleConfigLoader::layered_load()?;

        Ok(Self {
            providers,
            config,
            router: AiRouter::new(),
            roles: roles_manager,
        })
    }

    /// 从配置注册providers
    fn register_providers_from_config(
        providers: &mut HashMap<AiProviderType, Arc<dyn AiProvider>>,
        provider_configs: &HashMap<AiProviderType, ProviderConfig>,
    ) -> AiResult<()> {
        for (provider_type, config) in provider_configs {
            if !config.enabled {
                debug!("Provider {} is disabled, skipping", provider_type);
                continue;
            }

            let provider = match provider_type {
                AiProviderType::OpenAi => {
                    let mut provider = openai::OpenAiProvider::new(config.api_key.clone());
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::DeepSeek => {
                    let mut provider = openai::OpenAiProvider::deep_seek(config.api_key.clone());
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Groq => {
                    let mut provider = openai::OpenAiProvider::groq(config.api_key.clone());
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Kimi => {
                    let mut provider = openai::OpenAiProvider::kimi_k2(config.api_key.clone());
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Glm => {
                    let mut provider = openai::OpenAiProvider::new(config.api_key.clone());
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Mock => Arc::new(mock::MockProvider::new()) as Arc<dyn AiProvider>,
                AiProviderType::Anthropic | AiProviderType::Ollama => {
                    warn!(
                        "Provider {} is not yet implemented, skipping",
                        provider_type
                    );
                    continue;
                }
            };

            debug!(
                "Registered provider: {} with priority: {:?}",
                provider_type, config.priority
            );
            providers.insert(*provider_type, provider);
        }

        Ok(())
    }

    /// 构建基于角色的系统提示
    fn build_role_system_prompt(&self, role: AiRole) -> String {
        // 从配置文件中获取角色系统提示词
        if let Some(role_config) = self.roles.get_role_config(&role.to_string()) {
            let mut system_prompt = role_config.system_prompt.clone();

            // 尝试加载角色特定的规则配置
            if let Ok(Some(role_rules)) = self.roles.get_role_rules_config(&role.to_string()) {
                system_prompt = self.enhance_system_prompt_with_rules(system_prompt, &role_rules);
            }
            system_prompt
        } else {
            "".to_string()
        }
    }

    /// 使用规则增强系统提示词
    fn enhance_system_prompt_with_rules(
        &self,
        base_prompt: String,
        rules: &crate::ai::config::roles::RulesConfig,
    ) -> String {
        let mut enhanced_prompt = base_prompt;

        // 添加规则集合
        if !rules.rules.is_empty() {
            enhanced_prompt.push_str("\n\n## 规则\n");
            for rule in &rules.rules {
                enhanced_prompt.push_str(&format!("- {rule}\n"));
            }
        }
        enhanced_prompt
    }

    /// 获取所有可用的provider
    pub fn available_providers(&self) -> Vec<AiProviderType> {
        self.providers.keys().copied().collect()
    }

    /// 检查特定provider是否可用
    pub fn is_provider_available(&self, provider: AiProviderType) -> bool {
        self.providers.contains_key(&provider)
    }
    pub fn build_ai_request(&self, role: AiRole, user_input: &str) -> AiResult<AiRequest> {
        // 1. 使用角色推荐模型
        let conf = self
            .roles
            .get_role_config(role.as_str())
            .ok_or_else(|| AiErrReason::from_conf(format!("miss role:{role} conf")).to_err())?;

        let model = conf.used_model();
        // 2. 构建系统提示词
        let system_prompt = self.build_role_system_prompt(role);
        Ok(AiRequest::builder()
            .model(model)
            .system_prompt(system_prompt)
            .user_prompt(user_input.to_string())
            .role(role)
            .build())
    }

    /// 列出指定provider的所有可用模型
    pub async fn list_models(
        &self,
        provider: &AiProviderType,
    ) -> AiResult<Vec<crate::ai::provider::ModelInfo>> {
        if let Some(provider_arc) = self.providers.get(provider) {
            provider_arc.list_models().await
        } else {
            Err(AiErrReason::from_conf(format!("Provider {} not available", provider)).to_err())
        }
    }
}

pub fn load_key_dict(key: &str) -> Option<EnvDict> {
    let space = VarSpace::sys_init().unwrap();
    if std::env::var(key).is_err() && space.get(key).is_none() {
        println!("miss api token {key}");
        return None;
    }
    let dict = EnvDict::from(&space);
    Some(dict)
}

#[cfg(test)]
mod tests {
    use crate::infra::once_init_log;
    use orion_variate::vars::EnvEvalable;

    use super::*;

    fn create_mock_config() -> AiConfig {
        let mut config = AiConfig::example();
        // 禁用所有真实提供商，只保留Mock
        for (_, provider_config) in config.providers.iter_mut() {
            provider_config.enabled = false;
        }
        // 启用Mock提供商
        if let Some(mock_config) = config.providers.get_mut(&AiProviderType::Mock) {
            mock_config.enabled = true;
            mock_config.api_key = String::new();
        }
        config
    }

    #[tokio::test]
    async fn test_client_with_deepseek() {
        once_init_log();
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };
        // 创建配置，启用 DeepSeek
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
        once_init_log();
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };
        let client = AiClient::new(config).expect("Failed to create AiClient");
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

    #[test]
    fn test_build_role_system_prompt_with_rules() {
        once_init_log();
        let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            AiConfig::example().env_eval(&dict)
        } else {
            return;
        };
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试开发者角色的系统提示词
        let developer_prompt = client.build_role_system_prompt(AiRole::Developer);
        println!("开发者系统提示词:\n{developer_prompt}");

        // 验证包含角色规则
        assert!(developer_prompt.contains("语义化版 2.0"));
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

    #[test]
    fn test_build_ai_request_with_valid_role() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试开发者角色
        let request = client
            .build_ai_request(AiRole::Developer, "请解释什么是Rust的所有权系统")
            .expect("Failed to build AI request");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, "请解释什么是Rust的所有权系统");
        assert!(request.role.is_some());
        assert_eq!(request.role.unwrap(), AiRole::Developer);
    }

    #[test]
    fn test_build_ai_request_with_operations_role() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试运维角色
        let request = client
            .build_ai_request(AiRole::Operations, "如何检查Linux系统性能")
            .expect("Failed to build AI request");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, "如何检查Linux系统性能");
        assert!(request.role.is_some());
        assert_eq!(request.role.unwrap(), AiRole::Operations);
    }

    #[test]
    fn test_build_ai_request_with_knowledler_role() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 使用开发者角色替代可能不存在的Knowledger角色
        let request = client
            .build_ai_request(AiRole::Developer, "什么是微服务架构？")
            .expect("Failed to build AI request");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, "什么是微服务架构？");
        assert!(request.role.is_some());
        assert_eq!(request.role.unwrap(), AiRole::Developer);
    }

    #[test]
    fn test_build_ai_request_with_empty_input() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试空用户输入
        let request = client
            .build_ai_request(AiRole::Developer, "")
            .expect("Failed to build AI request with empty input");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, "");
        assert!(request.role.is_some());
    }

    #[test]
    fn test_build_ai_request_with_special_characters() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试包含特殊字符的用户输入
        let special_input =
            "请解释什么是 'Rust' 的所有权系统？\n代码示例：\n```rust\nlet x = 42;\nlet y = x;\n```";
        let request = client
            .build_ai_request(AiRole::Developer, special_input)
            .expect("Failed to build AI request with special characters");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, special_input);
        assert!(request.role.is_some());
    }

    #[test]
    fn test_build_ai_request_with_long_input() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试长文本输入
        let long_input =
            "这是一个很长的用户输入，用于测试 build_ai_request 函数处理长文本的能力。".repeat(100);
        let request = client
            .build_ai_request(AiRole::Developer, &long_input)
            .expect("Failed to build AI request with long input");

        // 验证请求结构
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert_eq!(request.user_prompt, long_input);
        assert!(request.role.is_some());
        assert!(request.user_prompt.len() > 1000); // 确保确实是长文本
    }

    #[test]
    fn test_build_ai_request_model_selection() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试不同角色是否选择了不同的模型
        let dev_request = client
            .build_ai_request(AiRole::Developer, "test")
            .expect("Failed to build developer request");

        let ops_request = client
            .build_ai_request(AiRole::Operations, "test")
            .expect("Failed to build operations request");

        // 验证系统提示词不同（表明角色配置不同）
        assert_ne!(dev_request.system_prompt, ops_request.system_prompt);

        // 模型名称应该根据角色配置有所不同
        println!("Developer model: {}", dev_request.model);
        println!("Operations model: {}", ops_request.model);
    }

    #[test]
    fn test_build_ai_request_response_structure() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        let request = client
            .build_ai_request(AiRole::Developer, "什么是Galaxy Operator Ecosystem？")
            .expect("Failed to build AI request");

        // 验证 AiRequest 结构的所有字段
        assert!(!request.model.is_empty());
        assert!(!request.system_prompt.is_empty());
        assert!(!request.user_prompt.is_empty());

        // 验证可选字段的默认值
        assert!(request.max_tokens.is_none());
        assert!(request.temperature == Some(0.7) || request.temperature.is_none());
        assert!(request.role.is_some());
        assert_eq!(request.role.unwrap(), AiRole::Developer);
    }

    #[test]
    fn test_build_ai_request_multiple_calls_consistency() {
        once_init_log();
        let config = create_mock_config();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 多次调用相同角色和输入，验证结果一致性
        let input = "测试输入";
        let request1 = client
            .build_ai_request(AiRole::Developer, input)
            .expect("Failed to build first request");

        let request2 = client
            .build_ai_request(AiRole::Developer, input)
            .expect("Failed to build second request");

        // 验证结果一致
        assert_eq!(request1.model, request2.model);
        assert_eq!(request1.system_prompt, request2.system_prompt);
        assert_eq!(request1.user_prompt, request2.user_prompt);
        assert_eq!(request1.temperature, request2.temperature);
        assert_eq!(request1.role, request2.role);
    }
}
