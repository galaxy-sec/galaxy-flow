use crate::config::{ProviderConfig, RoleConfigLoader, RoleConfigManager};
use crate::error::AiResult;
use crate::provider::{AiProvider, AiProviderType};
use crate::{AiConfig, AiRouter};
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::AiClient;
use crate::providers::{mock, openai};

/// AiClient 构建器
pub struct AiClientBuilder {
    providers: HashMap<AiProviderType, Arc<dyn AiProvider>>,
    config: AiConfig,
    router: AiRouter,
    roles: RoleConfigManager,
}

impl AiClientBuilder {
    /// 创建新的构建器
    pub fn new(config: AiConfig, role_file: Option<PathBuf>) -> AiResult<Self> {
        let mut providers: HashMap<AiProviderType, Arc<dyn AiProvider>> = HashMap::new();

        // 从配置注册provider
        Self::register_providers_from_config(&mut providers, &config.providers)?;

        // 初始化角色配置管理器 - 优先使用简化配置
        let roles_manager = RoleConfigLoader::layered_load(role_file)?;

        Ok(Self {
            providers,
            config,
            router: AiRouter::new(),
            roles: roles_manager,
        })
    }

    /// 构建 AiClient
    pub fn build(self) -> AiClient {
        AiClient {
            providers: self.providers,
            config: self.config,
            router: self.router,
            roles: self.roles,
        }
    }

    /// 从配置注册providers
    fn register_providers_from_config(
        providers: &mut HashMap<AiProviderType, Arc<dyn AiProvider>>,
        provider_configs: &HashMap<AiProviderType, ProviderConfig>,
    ) -> AiResult<()> {
        for (provider_type, config) in provider_configs {
            if !config.enabled {
                debug!("Provider {provider_type} is disabled, skipping");
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
                    debug!(
                        "Provider {provider_type} is not yet implemented, skipping"
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
}

/// 为 AiClient 提供构建相关的便利方法
impl AiClient {
    /// 创建AiClient（简化版本，无Thread支持）
    pub fn new(config: AiConfig, role_file: Option<PathBuf>) -> AiResult<Self> {
        AiClientBuilder::new(config, role_file).map(|builder| builder.build())
    }
}
