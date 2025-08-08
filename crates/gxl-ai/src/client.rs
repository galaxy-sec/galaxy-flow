use crate::error::{AiError, Result};
use crate::models::{AiRequest, AiResponse, ModelProvider};
use reqwest::Client;
use serde_json::{json, Value};
use std::time::Duration;

/// GXL原生AI通信客户端
pub struct AiClient {
    client: Client,
    openai_key: String,
    claude_key: Option<String>,
}

impl AiClient {
    pub fn new(openai_key: String, claude_key: Option<String>) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            openai_key,
            claude_key,
        }
    }

    /// GXL原生核心：AI模型直接通信
    pub async fn query(&self, request: AiRequest) -> Result<AiResponse> {
        match request.model {
            ModelProvider::Gpt4o | ModelProvider::Gpt4oMini =>
                self.request_openai(request).await,
            ModelProvider::Claude35 | ModelProvider::ClaudeHaiku =>
                self.request_claude(request).await,
            ModelProvider::Ollama(model) =>
                self.request_ollama(request, &model).await,
        }
    }

    /// OpenAI API 原生通信
    async fn request_openai(&self, request: AiRequest) -> Result<AiResponse> {
        let payload = json!({
+            "model": request.model.to_string(),
+            "messages": [
+                {"role": "system", "content": request.system_prompt},
+                {"role": "user", "content": request.user_prompt}
+            ],
+            "max_tokens": request.max_tokens,
+            "temperature": request.temperature
+        });
++
++        let response = self.client
++            .post("https://api.openai.com/v1/chat/completions")
++            .header("Authorization", format!("Bearer {}", self.openai_key))
++            .header("Content-Type", "application/json")
++            .json(&payload)
++            .send()
++            .await
++            .map_err(|e| AiError::NetworkError(e.to_string()))?;
++
++        let data: Value = response.json()
++            .await
++            .map_err(|e| AiError::ParseError(e.to_string()))?;
++
++        let content = data["choices"][0]["message"]["content"]
++            .as_str()
++            .ok_or(AiError::ResponseParseError)?;
++
++        Ok(AiResponse {
+            content: content.to_string(),
+            model: request.model,
+            input_tokens: data["usage"]["prompt_tokens"].as_u64().unwrap_or(0),
+            output_tokens: data["usage"]["completion_tokens"].as_u64().unwrap_or(0),
+            confidence: 0.95, // GPT典型置信度
+        })
++    }
++
++    /// Claude 3.5 API 直接通信
++    async fn request_claude(&self, request: AiRequest) -> Result<AiResponse> {
++        let claude_key = self.claude_key
++            .as_ref()
++            .ok_or(AiError::ApiKeyMissing("Claude".to_string()))?;
++
++        let payload = json!({
++            "model": request.model.to_string(),
++            "max_tokens": request.max_tokens,
++            "messages": [
++                {"role": "user", "content": format!("{}\n\n{}",
++                    request.system_prompt,
++                    request.user_prompt)
++                }
++            ]
++        });
++
++        let response = self.client
++            .post("https://api.anthropic.com/v1/messages")
++            .header("x-api-key", claude_key)
++            .header("anthropic-version", "2023-06-01")
++            .header("Content-Type", "application/json")
++            .json(&payload)
++            .send()
++            .await
++            .map_err(|e| AiError::NetworkError(e.to_string()))?;
++
++        let data: Value = response.json()
++            .await
++            .map_err(|e| AiError::ParseError(e.to_string()))?;
++
++        let content = data["content"][0]["text"]
++            .as_str()
++            .ok_or(AiError::ResponseParseError)?;
++
++        Ok(AiResponse {
++            content: content.to_string(),
++            model: request.model,
++            input_tokens: data["usage"]["input_tokens"].as_u64().unwrap_or(0),
++            output_tokens: data["usage"]["output_tokens"].as_u64().unwrap_or(0),
++            confidence: 0.9,
