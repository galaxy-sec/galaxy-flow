use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;

use crate::ai::provider::AiProviderType;
use crate::ai::error::AiResult;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AiConfig {
    pub providers: HashMap<AiProviderType, ProviderConfig>,
    pub routing: RoutingRules,
    pub limits: UsageLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key_env: String,
    pub base_url: Option<String>,
    pub timeout: u64,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key_env: "OPENAI_API_KEY".to_string(),
            base_url: None,
            timeout: 30,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRules {
    pub simple: String,
    pub complex: String,
    pub free: String,
}

impl Default for RoutingRules {
    fn default() -> Self {
        Self {
            simple: "gpt-4o-mini".to_string(),
            complex: "gpt-4o".to_string(),
            free: "deepseek-coder".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub max_tokens_per_request: usize,
    pub commit_budget: usize,
    pub review_budget: usize,
    pub analysis_budget: usize,
}

impl Default for UsageLimits {
    fn default() -> Self {
        Self {
            max_tokens_per_request: 16000,
            commit_budget: 150,
            review_budget: 2000,
            analysis_budget: 4000,
        }
    }
}

impl AiConfig {
    pub fn load() -> AiResult<Self> {
        // 尝试从环境变量加载
        let config = Self::from_env().unwrap_or_default();

        // 未来可以从配置文件加载
        if let Ok(path) = env::var("GXL_AI_CONFIG") {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(config_file) = serde_yaml::from_str(&content) {
                    return Ok(config_file);
                }
            }
        }

        Ok(config)
    }

    pub fn get_api_key(&self, provider: AiProviderType) -> Option<String> {
        if let Some(config) = self.providers.get(&provider) {
            env::var(&config.api_key_env).ok()
        } else {
            match provider {
                AiProviderType::OpenAi => env::var("OPENAI_API_KEY").ok(),
                AiProviderType::Anthropic => env::var("CLAUDE_API_KEY").ok(),
                AiProviderType::Ollama => Some("ollama".to_string()), // 本地无需密钥
                AiProviderType::Mock => Some("mock".to_string()),
            }
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // 默认配置
        providers.insert(AiProviderType::OpenAi, ProviderConfig {
            api_key_env: "OPENAI_API_KEY".to_string(),
            ..Default::default()
        });

        providers.insert(AiProviderType::Anthropic, ProviderConfig {
            api_key_env: "CLAUDE_API_KEY".to_string(),
            ..Default::default()
        });

        providers.insert(AiProviderType::Ollama, ProviderConfig {
            api_key_env: "OLLAMA_MODEL".to_string(),
            base_url: Some("http://localhost:11434".to_string()),
            ..Default::default()
        });

        Self {
            providers,
            routing: RoutingRules::default(),
            limits: UsageLimits::default(),
        }
    }
}

impl AiConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            providers: Default::default(),
            routing: RoutingRules {
                simple: env::var("GXL_SIMPLE_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string()),
                complex: env::var("GXL_COMPLEX_MODEL").unwrap_or_else(|_| "gpt-4o".to_string()),
+                free: env::var("GXL_FREE_MODEL").unwrap_or_else(|_| "deepseek-coder".to_string()),
+            },
+            limits: UsageLimits {
+                max_tokens_per_request: 16000,
+                commit_budget: env::var("GXL_COMMIT_BUDGET")
+                    .ok()
+                    .and_then(|s| s.parse().ok())
+                    .unwrap_or(150),
+                review_budget: env::var("GXL_REVIEW_BUDGET")
+                    .ok()
+                    .and_then(|s| s
