use crate::ai::capabilities::AiRole;
use crate::ai::config::{RoleConfigLoader, RoleConfigManager};
use crate::ai::error::{AiError, AiResult};
use crate::ai::provider::{AiProvider, AiProviderType, AiRequest};
use crate::execution::VarSpace;
use async_trait::async_trait;
use orion_variate::vars::EnvDict;
use std::collections::HashMap;
use std::sync::Arc;

use super::provider::AiResponse;
use super::providers::openai::OpenAiProvider;
use super::{AiConfig, AiErrReason, AiResult as OriginalAiResult, AiRouter};

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
pub struct AiClient {
    providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    config: Arc<AiConfig>,
    router: AiRouter,
    role_config_manager: Arc<RoleConfigManager>,
}

#[async_trait]
impl AiClientTrait for AiClient {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let provider_type = self.router.select_provider(&request.model, &self.config);

        if let Some(provider) = self.providers.get(&provider_type) {
            provider.send_request(&request).await
        } else {
            Err(AiError::from(AiErrReason::NoProviderAvailable))
        }
    }

    /// 基于角色的智能请求处理 - 用户只需选择角色，系统自动选择推荐模型
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
        let _role_name = &role.to_string();

        // 1. 使用角色推荐模型
        let model = role.recommended_model();

        // 2. 构建系统提示词
        let system_prompt = self.build_role_system_prompt(role);

        let request = AiRequest::builder()
            .model(model)
            .system_prompt(system_prompt)
            .user_prompt(user_input.to_string())
            .build();

        // 3. 发送请求
        let mut response = self.send_request(request).await?;

        // 4. 在响应中添加角色信息
        response.content = format!("[角色: {}]\n\n{}", role.description(), response.content);

        Ok(response)
    }
}

impl AiClient {
    /// 创建AiClient（简化版本，无Thread支持）
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
        let role_config_manager =
            RoleConfigLoader::layered_load().unwrap_or_else(|_| RoleConfigManager::default());

        Ok(Self {
            providers,
            config: Arc::new(config),
            router: AiRouter::new(),
            role_config_manager: Arc::new(role_config_manager),
        })
    }

    /// 构建基于角色的系统提示
    fn build_role_system_prompt(&self, role: AiRole) -> String {
        // 从配置文件中获取角色系统提示词
        if let Some(role_config) = self.role_config_manager.get_role_config(&role.to_string()) {
            let mut system_prompt = role_config.system_prompt.clone();

            // 尝试加载角色特定的规则配置
            if let Ok(Some(role_rules)) = self
                .role_config_manager
                .get_role_rules_config(&role.to_string())
            {
                system_prompt = self.enhance_system_prompt_with_rules(system_prompt, &role_rules);
            } else {
                // 如果没有角色特定规则，尝试加载全局规则
                if let Ok(global_rules) = self.role_config_manager.load_global_rules_config() {
                    system_prompt =
                        self.enhance_system_prompt_with_rules(system_prompt, &global_rules);
                }
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
        assert!(developer_prompt.contains("【开发者规则】"));
        assert!(developer_prompt.contains("禁止行为：不生成包含硬编码密钥的代码"));
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
}
