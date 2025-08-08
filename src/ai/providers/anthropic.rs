use async_trait::async_trait;
use reqwest::{Client, header};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::ai::{provider::*, error::AiResult, AiProviderType};

#[derive(Debug, Serialize, Deserialize)]
struct ClaudeRequest {
    model: String,
    messages: Vec<Message>,
    max_tokens: Option<usize>,
    temperature: Option<f32>,
    system: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ClaudeResponse {
    content: Vec<Content>,
    model: String,
    usage: Usage,
}

#[derive(Debug, Deserialize)]
struct Content {
    text: String,
    #[serde(rename = "type")]
    content_type: String,
}

#[derive(Debug, Deserialize)]
struct Usage {
    input_tokens: usize,
    output_tokens: usize,
}

pub struct AnthropicProvider {
    client: Arc<Client>,
    api_key: String,
    base_url: String,
}

impl AnthropicProvider {
    pub fn new(api_key: String) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(35))
            .build()
            .unwrap_or_else(|_| Client::new());

        Self {
            client: Arc::new(client),
            api_key,
            base_url: "https://api.anthropic.com/v1/messages".to_string(),
        }
+    }
+}
+
+#[async_trait]
+impl AiProvider for AnthropicProvider {
+    fn provider_type(&self) -> AiProviderType {
+        AiProviderType::Anthropic
+    }
+
+    async fn is_model_available(&self, model: &str) -> bool {
+        ["claude-3-5-sonnet-20241022", "claude-3-haiku-20240307", "claude-3-opus-20240229"]
+            .contains(&model)
+    }
+
+    async fn list_models(&self) -> AiResult<Vec<ModelInfo>> {
+        Ok(vec![
+            ModelInfo {
+                name: "claude-3-5-sonnet-20241022".to_string(),
+                provider: AiProviderType::Anthropic,
+                max_tokens: 200_000,
+                supports_images: true,
+                supports_reasoning: true,
+                cost_per_1k_input: 0.003,
+                cost_per_1k_output: 0.015,
+            },
+            ModelInfo {
+                name: "claude-3-haiku-20240307".to_string(),
+                provider: AiProviderType::Anthropic,
+                max_tokens: 200_000,
+                supports_images: true,
+                supports_reasoning: false,
+                cost_per_1k_input: 0.00025,
+                cost_per_1k_output: 0.00125,
+            },
+        ])
+    }
+
+    async fn send_request(&self, request: &AiRequest) -> AiResult<AiResponse> {
+        let claude_request = ClaudeRequest {
+            model: request.model.clone(),
+            messages: vec![Message {
+                role: "user".to_string(),
+                content: request.user_prompt.clone(),
+            }],
+            max_tokens: request.max_tokens.or(Some(1000)),
+            temperature: request.temperature.or(Some(0.7)),
+            system: Some(request.system_prompt.clone()),
+        };
+
+        let response = self.client
+            .post(&self.base_url)
+            .header("x-api-key", &self.api_key)
+            .header("anthropic-version", "2023-06-01")
+            .header("content-type", "application/json")
+            .json(&claude_request)
+            .send()
+            .await?;
+
+        let claude_response = response.json::<ClaudeResponse>().await?;
+
+        let content = claude_response.content
+            .first()
+            .map(|c| c.text.clone())
+            .unwrap_or_default();
+
+        Ok(AiResponse {
+            content,
+            model: claude_response.model,
+            usage: UsageInfo {
+                prompt_tokens: claude_response.usage.input_tokens,
+                completion_tokens: claude_response.usage.output_tokens,
+                total_tokens: claude_response.usage.input_tokens + claude_response.usage.output_tokens,
+                estimated_cost: self.estimate_cost(
+                    &request.model,
+                    claude_response.usage.input_tokens,
+                    claude_response.usage.output_tokens,
+                ),
+            },
+            finish_reason: Some("stop".to_string()),
+
