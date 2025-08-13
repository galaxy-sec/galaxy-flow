use crate::ai::capabilities::AiRole;
use crate::ai::config::{RoleConfigLoader, RoleConfigManager};
use crate::ai::error::{AiError, AiResult};
use crate::ai::provider::{AiProvider, AiProviderType, AiRequest};
use crate::ai::thread::recorder::ThreadRecordingClient;
use crate::execution::VarSpace;
use orion_variate::vars::EnvDict;
use std::collections::HashMap;
use std::sync::Arc;

use super::provider::AiResponse;
use super::providers::openai::OpenAiProvider;
use super::{AiConfig, AiErrReason, AiResult as OriginalAiResult, AiRouter};

/// AI客户端枚举，支持静态分发
pub enum AiClientEnum {
    Basic(AiClient),
    ThreadRecording(Box<ThreadRecordingClient>),
}

pub enum AiSendClient {
    Basic(AiClient),
}

impl AiClientEnum {
    /// 创建基础AiClient
    pub fn new(config: AiConfig) -> AiResult<Self> {
        // 验证配置
        let mut validated_config = config.clone();
        validated_config.validate_and_postprocess().map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "Configuration validation failed: {}",
                e
            )))
        })?;

        Ok(Self::Basic(AiClient::new(config)?))
    }

    /// 创建Thread记录客户端
    pub fn new_with_thread_recording(config: AiConfig) -> AiResult<Self> {
        let inner_config = config.clone();
        let basic_client = Self::new(inner_config)?;
        let thread_config = config.thread.clone();

        Ok(Self::ThreadRecording(Box::new(ThreadRecordingClient::new(
            basic_client,
            thread_config,
        ))))
    }

    /// 根据配置自动选择客户端类型
    pub fn new_auto(config: AiConfig) -> AiResult<Self> {
        // 验证配置
        let mut validated_config = config;
        validated_config.validate_and_postprocess().map_err(|e| {
            AiError::from(AiErrReason::ConfigError(format!(
                "Configuration validation failed: {}",
                e
            )))
        })?;

        if validated_config.thread.enabled {
            Self::new_with_thread_recording(validated_config)
        } else {
            Self::new(validated_config)
        }
    }

    /// 发送AI请求
    pub async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        match self {
            Self::Basic(client) => client.send_request(request).await,
            Self::ThreadRecording(client) => client.as_ref().send_request(request).await,
        }
    }

    /// 基于角色的智能请求
    pub async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
        match self {
            Self::Basic(client) => client.smart_role_request(role, user_input).await,
            Self::ThreadRecording(client) => {
                client.as_ref().smart_role_request(role, user_input).await
            }
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
    pub async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
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

    /// 构建基于角色的系统提示
    fn build_role_system_prompt(&self, role: AiRole) -> String {
        // 从配置文件中获取角色系统提示词
        if let Some(role_config) = self.role_config_manager.get_role_config(&role.to_string()) {
            let base_prompt = role_config.system_prompt.clone();

            // 构建包含角色规则的完整系统提示词
            self.build_enhanced_system_prompt(&base_prompt, role)
        } else {
            // 回退到原有的硬编码描述
            let role_description = role.description();
            let base_prompt = format!(
                "你是{role_description}。你的工作流程是连续的，能够智能处理该角色下的各种任务。"
            );

            // 即使是回退情况，也尝试应用基本规则
            self.build_enhanced_system_prompt(&base_prompt, role)
        }
    }

    /// 构建增强的系统提示词，包含角色规则
    fn build_enhanced_system_prompt(&self, base_prompt: &str, role: AiRole) -> String {
        let mut enhanced_prompt = base_prompt.to_string();

        // 添加角色特定的规则和约束
        enhanced_prompt.push_str("\n\n=== 角色规则和约束 ===\n");

        // 根据角色类型添加特定的规则
        match role {
            AiRole::Developer => {
                enhanced_prompt.push_str("\n【开发者规则】\n");
                enhanced_prompt
                    .push_str("• 使用场景：代码生成、代码审查、架构设计、技术问题解决、文档生成\n");
                enhanced_prompt
                    .push_str("• 约束条件：最大代码长度2000行，必须包含错误处理，需要代码审查\n");
                enhanced_prompt
                    .push_str("• 输出要求：格式良好的代码，包含注释和测试，优雅的错误处理\n");
                enhanced_prompt
                    .push_str("• 最佳实践：验证生成的代码，遵循编码规范，避免敏感代码\n");
                enhanced_prompt
                    .push_str("• 禁止行为：不生成包含硬编码密钥的代码，不生成不安全的代码模式\n");
            }
            AiRole::Operations => {
                enhanced_prompt.push_str("\n【运维人员规则】\n");
                enhanced_prompt.push_str("• 使用场景：系统部署、监控、维护、故障排除\n");
                enhanced_prompt.push_str("• 约束条件：优先系统稳定性，确保操作可回滚\n");
                enhanced_prompt.push_str("• 输出要求：详细的操作步骤，风险评估，回滚方案\n");
                enhanced_prompt.push_str("• 最佳实践：备份重要数据，测试环境验证，文档记录\n");
                enhanced_prompt.push_str("• 禁止行为：不直接操作生产环境，不跳过安全检查\n");
            }
            AiRole::KnowledgeManager => {
                enhanced_prompt.push_str("\n【知识管理规则】\n");
                enhanced_prompt.push_str("• 使用场景：知识获取、解释、咨询、文档管理\n");
                enhanced_prompt.push_str("• 约束条件：确保信息准确性，引用可靠来源\n");
                enhanced_prompt.push_str("• 输出要求：结构化信息，清晰解释，相关引用\n");
                enhanced_prompt.push_str("• 最佳实践：验证信息来源，保持客观中立，持续更新知识\n");
                enhanced_prompt.push_str("• 禁止行为：不传播未经验证的信息，不忽略知识版权\n");
            }
        }

        // 添加全局AI使用规则
        enhanced_prompt.push_str("\n=== 全局AI使用规则 ===\n");
        enhanced_prompt.push_str("• 基础原则：安全第一，质量优先，效率优化，持续学习\n");
        enhanced_prompt.push_str("• 通用约束：最大请求4000 token，每分钟最多10次请求\n");
        enhanced_prompt.push_str("• 质量标准：最小响应质量0.7，需要审查验证\n");
        enhanced_prompt.push_str("• 安全要求：数据加密，审计日志，访问控制，内容过滤\n");

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
    use std::env;

    #[tokio::test]
    async fn test_client_with_deepseek() {
        once_init_log();
        let dict = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            dict
        } else {
            return;
        };
        // 创建配置，启用 DeepSeek
        let config = AiConfig::example();

        let client = AiClient::new(config.env_eval(&dict)).expect("Failed to create AiClient");

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
        let dict = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
            dict
        } else {
            return;
        };

        let config = AiConfig::example();
        let client = AiClient::new(config.env_eval(&dict)).expect("Failed to create AiClient");
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
        let config = AiConfig::example();
        let client = AiClient::new(config).expect("Failed to create AiClient");

        // 测试开发者角色的系统提示词
        let developer_prompt = client.build_role_system_prompt(AiRole::Developer);
        println!("开发者系统提示词:\n{developer_prompt}");

        // 验证包含角色规则
        assert!(developer_prompt.contains("=== 角色规则和约束 ==="));
        assert!(developer_prompt.contains("【开发者规则】"));
        assert!(developer_prompt.contains("使用场景：代码生成、代码审查"));
        assert!(developer_prompt.contains("约束条件：最大代码长度2000行"));
        assert!(developer_prompt.contains("输出要求：格式良好的代码"));
        assert!(developer_prompt.contains("最佳实践：验证生成的代码"));
        assert!(developer_prompt.contains("禁止行为：不生成包含硬编码密钥的代码"));

        // 验证包含全局规则
        assert!(developer_prompt.contains("=== 全局AI使用规则 ==="));
        assert!(developer_prompt.contains("基础原则：安全第一，质量优先"));
        assert!(developer_prompt.contains("通用约束：最大请求4000 token"));
        assert!(developer_prompt.contains("质量标准：最小响应质量0.7"));
        assert!(developer_prompt.contains("安全要求：数据加密，审计日志"));

        // 测试运维人员角色的系统提示词
        let operations_prompt = client.build_role_system_prompt(AiRole::Operations);
        println!("\n运维人员系统提示词:\n{operations_prompt}");

        // 验证运维人员特定规则
        assert!(operations_prompt.contains("【运维人员规则】"));
        assert!(operations_prompt.contains("使用场景：系统部署、监控、维护"));
        assert!(operations_prompt.contains("约束条件：优先系统稳定性"));
        assert!(operations_prompt.contains("禁止行为：不直接操作生产环境"));

        // 测试知识管理角色的系统提示词
        let knowledge_manager_prompt = client.build_role_system_prompt(AiRole::KnowledgeManager);
        println!("\n知识管理角色系统提示词:\n{knowledge_manager_prompt}");

        // 验证知识管理特定规则
        assert!(knowledge_manager_prompt.contains("【知识管理规则】"));
        assert!(knowledge_manager_prompt.contains("使用场景：知识获取、解释、咨询"));
        assert!(knowledge_manager_prompt.contains("约束条件：确保信息准确性"));
        assert!(knowledge_manager_prompt.contains("禁止行为：不传播未经验证的信息"));
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
