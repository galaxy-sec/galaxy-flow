use crate::config::{ProviderConfig, RoleConfigLoader};
use crate::error::AiResult;
use crate::provider::{AiProvider, AiProviderType};
use crate::{AiConfig, AiRouter};
use log::debug;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use super::AiClient;
use crate::providers::{mock, openai};

use getset::{Getters, MutGetters, Setters, WithSetters};
#[derive(Clone, Debug, Getters, Setters, WithSetters, MutGetters)]
#[getset(get = "pub", set = "pub", get_mut = "pub", set_with = "pub")]
/// AiClient 构建器
pub struct AiClientBuilder {
    config: AiConfig,
    timout: u64,
    role_file: Option<PathBuf>,
}

impl AiClientBuilder {
    /// 创建新的构建器
    pub fn new(config: AiConfig) -> Self {
        Self {
            config,
            timout: 30,
            role_file: None,
        }
    }
    pub fn with_role(self, role_file: PathBuf) -> Self {
        self.with_role_file(Some(role_file))
    }

    /// 构建 AiClient
    pub fn build(self) -> AiResult<AiClient> {
        let mut providers: HashMap<AiProviderType, Arc<dyn AiProvider>> = HashMap::new();
        // 从配置注册provider
        Self::register_providers_from_config(&mut providers, &self.config.providers, self.timout)?;

        // 初始化角色配置管理器 - 优先使用简化配置
        let roles_manager = RoleConfigLoader::layered_load(self.role_file.clone())?;
        Ok(AiClient {
            providers,
            config: self.config,
            router: AiRouter::new(),
            roles: roles_manager,
        })
    }

    /// 从配置注册providers
    fn register_providers_from_config(
        providers: &mut HashMap<AiProviderType, Arc<dyn AiProvider>>,
        provider_configs: &HashMap<AiProviderType, ProviderConfig>,
        timeout_sec: u64,
    ) -> AiResult<()> {
        for (provider_type, config) in provider_configs {
            if !config.enabled {
                debug!("Provider {provider_type} is disabled, skipping");
                continue;
            }

            let provider = match provider_type {
                AiProviderType::OpenAi => {
                    let mut provider =
                        openai::OpenAiProvider::new(config.api_key.clone(), timeout_sec);
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::DeepSeek => {
                    let mut provider =
                        openai::OpenAiProvider::deep_seek(config.api_key.clone(), timeout_sec);
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Groq => {
                    let mut provider =
                        openai::OpenAiProvider::groq(config.api_key.clone(), timeout_sec);
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Kimi => {
                    let mut provider =
                        openai::OpenAiProvider::kimi_k2(config.api_key.clone(), timeout_sec);
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Glm => {
                    let mut provider =
                        openai::OpenAiProvider::new(config.api_key.clone(), timeout_sec);
                    if let Some(base_url) = &config.base_url {
                        provider = provider.with_base_url(base_url.clone());
                    }
                    Arc::new(provider) as Arc<dyn AiProvider>
                }
                AiProviderType::Mock => Arc::new(mock::MockProvider::new()) as Arc<dyn AiProvider>,
                AiProviderType::Anthropic | AiProviderType::Ollama => {
                    debug!("Provider {provider_type} is not yet implemented, skipping");
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
