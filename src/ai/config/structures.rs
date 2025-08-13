use orion_common::serde::Yamlable;
use orion_error::{ToStructError, UvsConfFrom, UvsResFrom};
use orion_variate::vars::{EnvDict, EnvEvalable};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::env::home_dir;
use std::path::PathBuf;

use crate::ai::provider::AiProviderType;
use crate::const_val::gxl_const::AI_CONF_FILE;
use crate::{ExecReason, ExecResult};

/// AI配置主结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub providers: HashMap<AiProviderType, ProviderConfig>,
    pub routing: RoutingRules,
    pub limits: UsageLimits,
    pub file_config: Option<FileConfig>,
}

impl EnvEvalable<AiConfig> for AiConfig {
    fn env_eval(self, dict: &EnvDict) -> Self {
        let providers = Self::eval_providers_hashmap(self.providers, dict);
        let routing = self.routing.env_eval(dict);
        let limits = self.limits.env_eval(dict);
        let file_config = self.file_config.map(|fc| fc.env_eval(dict));

        Self {
            providers,
            routing,
            limits,
            file_config,
        }
    }
}

impl AiConfig {
    fn eval_providers_hashmap(
        providers: HashMap<AiProviderType, ProviderConfig>,
        dict: &EnvDict,
    ) -> HashMap<AiProviderType, ProviderConfig> {
        providers
            .into_iter()
            .map(|(k, v)| (k, v.env_eval(dict)))
            .collect()
    }
    pub fn galaxy_load(dict: &EnvDict) -> ExecResult<Self> {
        let galaxy_dir = home_dir()
            .ok_or_else(|| ExecReason::from_res("Cannot find home directory".into()))?
            .join(".galaxy");
        let gal_ai_conf = galaxy_dir.join(AI_CONF_FILE);
        let prj_ai_conf = PathBuf::from("./_gal").join(AI_CONF_FILE);
        let ai_conf = if prj_ai_conf.exists() {
            prj_ai_conf
        } else if gal_ai_conf.exists() {
            gal_ai_conf
        } else {
            return ExecReason::from_conf("miss ai config".to_string()).err_result();
        };
        let conf = AiConfig::from_yml(&ai_conf)
            .map_err(|e| ExecReason::from_conf(format!("ai_conf :{e}")))?;
        Ok(conf.env_eval(dict))
    }

    /// 提供 deepseek,openai, glm,kimi 的访问配置。
    /// TOKEN 使用环量变量表示
    pub fn example() -> Self {
        let mut providers = HashMap::new();

        // OpenAI 配置
        providers.insert(
            AiProviderType::OpenAi,
            ProviderConfig {
                enabled: true,
                api_key: "${SEC_OPENAI_API_KEY}".to_string(),
                base_url: Some("https://api.openai.com/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(1),
            },
        );

        // DeepSeek 配置
        providers.insert(
            AiProviderType::DeepSeek,
            ProviderConfig {
                enabled: true,
                api_key: "${SEC_DEEPSEEK_API_KEY}".to_string(),
                base_url: Some("https://api.deepseek.com/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(2),
            },
        );

        // GLM 配置
        providers.insert(
            AiProviderType::Glm,
            ProviderConfig {
                enabled: true,
                api_key: "${SEC_GLM_API_KEY}".to_string(),
                base_url: Some("https://open.bigmodel.cn/api/paas/v4".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(3),
            },
        );

        // Kimi 配置
        providers.insert(
            AiProviderType::Kimi,
            ProviderConfig {
                enabled: true,
                api_key: "${SEC_KIMI_API_KEY}".to_string(),
                base_url: Some("https://api.moonshot.cn/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(4),
            },
        );

        Self {
            providers,
            routing: RoutingRules::default(),
            limits: UsageLimits::default(),
            file_config: None,
        }
    }

    /// 从环境变量加载配置（传统方式）
    pub fn from_env() -> Self {
        let mut providers = HashMap::new();

        // 初始化默认的ProviderConfig
        providers.insert(
            AiProviderType::OpenAi,
            ProviderConfig {
                enabled: true,
                api_key: "${OPENAI_API_KEY}".to_string(),
                base_url: Some("https://api.openai.com/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(1),
            },
        );

        providers.insert(
            AiProviderType::DeepSeek,
            ProviderConfig {
                enabled: true,
                api_key: "${DEEPSEEK_API_KEY}".to_string(),
                base_url: Some("https://api.deepseek.com/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(2),
            },
        );

        providers.insert(
            AiProviderType::Groq,
            ProviderConfig {
                enabled: false,
                api_key: "${GROQ_API_KEY}".to_string(),
                base_url: Some("https://api.groq.com/openai/v1".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(3),
            },
        );

        providers.insert(
            AiProviderType::Mock,
            ProviderConfig {
                enabled: true,
                api_key: "mock".to_string(),
                base_url: None,
                timeout: 30,
                model_aliases: None,
                priority: Some(999),
            },
        );

        providers.insert(
            AiProviderType::Anthropic,
            ProviderConfig {
                enabled: false,
                api_key: "${CLAUDE_API_KEY}".to_string(),
                base_url: None,
                timeout: 30,
                model_aliases: None,
                priority: Some(4),
            },
        );

        providers.insert(
            AiProviderType::Ollama,
            ProviderConfig {
                enabled: false,
                api_key: "${OLLAMA_MODEL}".to_string(),
                base_url: Some("http://localhost:11434".to_string()),
                timeout: 30,
                model_aliases: None,
                priority: Some(5),
            },
        );

        Self {
            providers,
            routing: RoutingRules::default(),
            limits: UsageLimits::default(),
            file_config: None,
        }
    }

    /// 获取API密钥
    pub fn get_api_key(&self, provider: AiProviderType) -> Option<String> {
        if let Some(config) = self.providers.get(&provider) {
            if config.enabled {
                // 直接返回 api_key 值，变量替换已经在 env_eval 中实现
                Some(config.api_key.clone())
            } else {
                None
            }
        } else {
            match provider {
                AiProviderType::OpenAi => std::env::var("OPENAI_API_KEY").ok(),
                AiProviderType::Anthropic => std::env::var("CLAUDE_API_KEY").ok(),
                AiProviderType::Ollama => Some("ollama".to_string()), // 本地无需密钥
                AiProviderType::Mock => Some("mock".to_string()),
                AiProviderType::DeepSeek => std::env::var("DEEPSEEK_API_KEY").ok(),
                AiProviderType::Groq => std::env::var("GROQ_API_KEY").ok(),
                AiProviderType::Kimi => std::env::var("KIMI_API_KEY").ok(),
                AiProviderType::Glm => std::env::var("GLM_API_KEY").ok(),
            }
        }
    }
}

/// 文件配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileConfig {
    pub enabled: bool,
    pub override_env: bool,
    pub version: String,
    #[serde(skip)]
    pub config_path: PathBuf,
}

impl EnvEvalable<FileConfig> for FileConfig {
    fn env_eval(self, dict: &EnvDict) -> Self {
        Self {
            enabled: self.enabled,
            override_env: self.override_env,
            version: self.version.env_eval(dict),
            config_path: self.config_path,
        }
    }
}

/// 提供商配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub api_key: String,
    pub base_url: Option<String>,
    pub timeout: u64,
    pub model_aliases: Option<HashMap<String, String>>,
    pub priority: Option<u32>,
}

impl EnvEvalable<ProviderConfig> for ProviderConfig {
    fn env_eval(self, dict: &EnvDict) -> Self {
        let api_key = self.api_key.env_eval(dict);
        let base_url = Self::eval_base_url(self.base_url, dict);
        let model_aliases = Self::eval_model_aliases(self.model_aliases, dict);

        Self {
            enabled: self.enabled,
            api_key,
            base_url,
            timeout: self.timeout,
            model_aliases,
            priority: self.priority,
        }
    }
}

impl ProviderConfig {
    fn eval_base_url(base_url: Option<String>, dict: &EnvDict) -> Option<String> {
        base_url.map(|url| url.env_eval(dict))
    }

    fn eval_model_aliases(
        model_aliases: Option<HashMap<String, String>>,
        dict: &EnvDict,
    ) -> Option<HashMap<String, String>> {
        model_aliases.map(|aliases| {
            aliases
                .into_iter()
                .map(|(k, v)| (k, v.env_eval(dict)))
                .collect()
        })
    }
}

/// 路由规则结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRules {
    pub simple: String,
    pub complex: String,
    pub free: String,
}

impl EnvEvalable<RoutingRules> for RoutingRules {
    fn env_eval(self, dict: &EnvDict) -> Self {
        Self {
            simple: self.simple.env_eval(dict),
            complex: self.complex.env_eval(dict),
            free: self.free.env_eval(dict),
        }
    }
}

/// 使用限制结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageLimits {
    pub review_budget: usize,
    pub analysis_budget: usize,
}

impl EnvEvalable<UsageLimits> for UsageLimits {
    fn env_eval(self, _dict: &EnvDict) -> Self {
        Self {
            review_budget: self.review_budget,
            analysis_budget: self.analysis_budget,
        }
    }
}

/// Default 实现们
impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: "${OPENAI_API_KEY}".to_string(),
            base_url: None,
            timeout: 30,
            model_aliases: None,
            priority: None,
        }
    }
}

impl Default for RoutingRules {
    fn default() -> Self {
        Self {
            simple: "gpt-4o-mini".to_string(),
            complex: "gpt-4o".to_string(),
            free: "deepseek-chat".to_string(),
        }
    }
}

impl Default for UsageLimits {
    fn default() -> Self {
        Self {
            review_budget: 2000,
            analysis_budget: 4000,
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self::from_env()
    }
}
