use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::ai::{provider::AiProviderType, error::AiResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub providers: HashMap<AiProviderType, ProviderConfig>,
    pub routing: RoutingRules,
    pub limits: UsageLimits,
    pub sensitive: SensitiveFilter,
    pub global: GlobalSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub enabled: bool,
    pub models: Vec<String>,
    pub api_key_env: String,
    pub base_url: Option<String>,
    pub timeout: u64,
    pub max_retries: u32,
}

impl Default for ProviderConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            models: vec![],
            api_key_env: "OPENAI_API_KEY".to_string(),
            base_url: None,
            timeout: 30,
            max_retries: 3,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRules {
    pub simple: String,
    pub complex: String,
    pub free: String,
    pub fail_over: Vec<String>,
    pub priority: Vec<AiProviderType>,
+}
+
+impl Default for RoutingRules {
+    fn default() -> Self {
+        Self {
+            simple: "gpt-4o-mini".to_string(),
+            complex: "claude-3-5-sonnet".to_string(),
+            free: "ollama/deepseek-coder".to_string(),
+            fail_over: vec!["gpt-4o".to_string(), "claude-3.5-sonnet".to_string()],
+            priority: vec![
+                AiProviderType::OpenAi,
+                AiProviderType::Anthropic,
+                AiProviderType::Ollama,
+            ],
+        }
+    }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct UsageLimits {
+    pub max_tokens_per_request: usize,
+    pub max_tokens_per_day: Option<usize>,
+    pub sensitive_filter: bool,
+    pub token_budgets: HashMap<String, usize>,
+}
+
+impl Default for UsageLimits {
+    fn default() -> Self {
+        let mut budgets = HashMap::new();
+        budgets.insert("commit".to_string(), 150);
+        budgets.insert("review".to_string(), 2000);
+        budgets.insert("analysis".to_string(), 4000);
+
+        Self {
+            max_tokens_per_request: 16000,
+            max_tokens_per_day: None,
+            sensitive_filter: true,
+            token_budgets: budgets,
+        }
+    }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct SensitiveFilter {
+    pub enabled: bool,
+    pub api_key_patterns: Vec<String>,
+    pub email_patterns: Vec<String>,
+    pub private_key_patterns: Vec<String>,
+}
+
+impl Default for SensitiveFilter {
+    fn default() -> Self {
+        Self {
+            enabled: true,
+            api_key_patterns: vec![
+                r"(?i)api[_-]?key".to_string(),
+                r"(?i)secret[_-]?key".to_string(),
+                r"(?i)access[_-]?token".to_string(),
+            ],
+            email_patterns: vec![r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b".to_string()],
+            private_key_patterns: vec![
+                r"-----BEGIN PRIVATE KEY-----".to_string(),
+                r"-----BEGIN RSA PRIVATE KEY-----".to_string(),
+            ],
+        }
+    }
+}
+
+#[derive(Debug, Clone, Serialize, Deserialize)]
+pub struct GlobalSettings {
+    pub default_provider: AiProviderType,
+    pub default_timeout: u64,
+    pub enable_streaming: bool,
+    pub interactive_mode: bool,
+    pub log_requests: bool,
+    pub config_path: Option<String>,
+}
+
+impl Default for GlobalSettings {
+    fn default() -> Self {
+        Self {
+            default_provider: AiProviderType::OpenAi,
+            default_timeout: 30,
+            enable_streaming: false,
+            interactive_mode: true,
+            log_requests: false,
+            config_path: None,
+        }
+    }
+}

impl Default for AiConfig {
    fn default() -> Self {
        let mut providers = HashMap::new();

        // 默认OpenAI配置
        providers.insert(AiProviderType::
