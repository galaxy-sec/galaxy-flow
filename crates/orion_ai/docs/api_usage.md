# Orion AI API 使用文档

## 概述

Orion AI 提供了统一的接口来与多个 AI 服务提供商进行交互。本文档详细介绍如何使用该库的各种功能。

## 基础设置

### 1. 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
orion_ai = { path = "../crates/orion_ai" }
tokio = { version = "1.0", features = ["full"] }
```

### 2. 环境变量配置

```bash
# OpenAI 配置
export OPENAI_API_KEY="your-openai-api-key"
export OPENAI_BASE_URL="https://api.openai.com/v1"

# DeepSeek 配置
export DEEPSEEK_API_KEY="your-deepseek-api-key"
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"

# Groq 配置
export GROQ_API_KEY="your-groq-api-key"
export GROQ_BASE_URL="https://api.groq.com/openai/v1"

# 其他提供商...
```

## 基础使用

### 1. 创建客户端

```rust
use orion_ai::{AiClient, AiConfig, AiRequest};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量加载配置
    let config = AiConfig::from_env()?;
    
    // 创建客户端
    let client = AiClient::new(config, None)?;
    
    Ok(())
}
```

### 2. 发送简单请求

```rust
use orion_ai::{AiRequest, AiRoleID};

// 构建请求
let request = AiRequest::builder()
    .model("gpt-4o-mini")
    .system_prompt("你是一个有用的助手")
    .user_prompt("请解释什么是Rust编程语言")
    .temperature(0.7)
    .max_tokens(500)
    .build();

// 发送请求
let response = client.send_request(request).await?;
println!("AI响应: {}", response.content);
println!("使用的模型: {}", response.model);
println!("Token使用: {:?}", response.usage);
```

### 3. 使用角色系统

```rust
// 创建角色
let role = AiRoleID::new("developer");

// 发送基于角色的请求
let response = client.smart_role_request(&role, "请帮我优化这段代码").await?;
println!("角色响应: {}", response.content);
```

## 高级功能

### 1. 配置管理

#### 从文件加载配置

```yaml
# ai.yml
providers:
  openai:
    enabled: true
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    timeout: 30
    priority: 1
    model_aliases:
      gpt4: "gpt-4"
      gpt4-mini: "gpt-4o-mini"
  
  deepseek:
    enabled: true
    api_key: "${DEEPSEEK_API_KEY}"
    base_url: "https://api.deepseek.com/v1"
    timeout: 30
    priority: 2

routing:
  simple: "openai"
  complex: "openai"
  free: "deepseek"

limits:
  review_budget: 10000
  analysis_budget: 50000

thread:
  enabled: true
  storage_path: "./threads"
  filename_template: "thread_{timestamp}_{role}.md"
  min_summary_length: 200
  max_summary_length: 500
  summary_keywords: ["总结", "结论", "要点"]
  inform_ai: true
  inform_message: "这是一个延续的对话，请参考前面的上下文。"
```

#### 在代码中加载配置

```rust
use orion_ai::config::ConfigLoader;

let config = ConfigLoader::from_file("ai.yml")?;
let client = AiClient::new(config, None)?;
```

### 2. 角色配置

#### 角色配置文件

```yaml
# ai-roles.yml
default_role:
  id: galactiward
default_model: deepseek-chat

roles:
  operations:
    name: operations
    description: 专注于系统运维的专家
    system_prompt: |
      你是一个专业的运维专家，擅长诊断系统问题和解决问题。
      请提供清晰、可操作的建议。
    used_model: gpt-4o
    rules_path: ai-rules/operations
  
  developer:
    name: developer
    description: 专注于代码开发的技术专家
    system_prompt: |
      你是一个专业的开发者，擅长高质量的代码实现、系统设计和技术问题解决。
      请提供最佳的实践建议。
    used_model: deepseek-coder
    rules_path: ai-rules/developer
  
  galactiward:
    name: galactiward
    description: 专注于Galaxy生态专家
    system_prompt: |
      通过Galaxy资料，解决Galaxy问题。
    used_model: deepseek-chat
    rules_path: ai-rules/galactiward
```

#### 使用角色

```rust
use orion_ai::{AiRoleID, AiClient};

// 加载角色配置
let role_file = Some(std::path::PathBuf::from("ai-roles.yml"));
let client = AiClient::new(config, role_file)?;

// 使用特定角色
let operations_role = AiRoleID::new("operations");
let response = client.smart_role_request(&operations_role, "服务器CPU使用率过高").await?;
```

### 3. 线程记录功能

#### 启用线程记录

```rust
use orion_ai::factory::AiClientEnum;

// 创建带线程记录的客户端
let config = AiConfig::from_env()?;
config.thread.enabled = true; // 启用线程记录

let client_enum = AiClientEnum::new_with_thread_recording(config)?;

// 发送请求（会自动记录）
let request = AiRequest::builder()
    .model("gpt-4o")
    .user_prompt("这是一个测试消息")
    .role(AiRoleID::new("test"))
    .build();

let response = client_enum.send_request(request).await?;
```

#### 自定义线程配置

```rust
use orion_ai::config::ThreadConfig;
use std::path::PathBuf;

let thread_config = ThreadConfig {
    enabled: true,
    storage_path: PathBuf::from("./custom_threads"),
    filename_template: "chat_{timestamp}_{role}_{id}.md".to_string(),
    min_summary_length: 100,
    max_summary_length: 300,
    summary_keywords: vec
!["要点".to_string(), "总结".to_string(), "关键".to_string()],
    inform_ai: true,
    inform_message: "这是连续对话的一部分，请保持上下文连贯性。".to_string(),
};

let mut config = AiConfig::from_env()?;
config.thread = thread_config;

let client = AiClient::new(config, None)?;
```

### 4. 多提供商管理

#### 检查可用提供商

```rust
use orion_ai::provider::AiProviderType;

// 获取所有可用提供商
let available_providers = client.available_providers();
println!("可用提供商: {:?}", available_providers);

// 检查特定提供商是否可用
let is_openai_available = client.is_provider_available(AiProviderType::OpenAi);
println!("OpenAI 可用: {}", is_openai_available);
```

#### 列出模型

```rust
use orion_ai::provider::AiProviderType;

// 列出特定提供商的模型
let provider_type = AiProviderType::OpenAi;
let models = client.list_models(&provider_type).await?;

for model in models {
    println!("模型: {}", model.name);
    println!("提供商: {}", model.provider);
    println!("最大Token: {}", model.max_tokens);
    println!("支持图片: {}", model.supports_images);
    println!("支持推理: {}", model.supports_reasoning);
    println!("输入成本: ${}/1k tokens", model.cost_per_1k_input);
    println!("输出成本: ${}/1k tokens", model.cost_per_1k_output);
    println!("---");
}
```

### 5. 自定义路由

#### 创建自定义路由

```rust
use orion_ai::{AiRouter, AiProviderType};

let mut router = AiRouter::new();

// 注册自定义路由规则
router.register_rule("coding".to_string(), AiProviderType::DeepSeek);
router.register_rule("creative".to_string(), AiProviderType::OpenAi);

// 使用自定义路由
let provider = router.select_provider("coding-task", &config);
println!("选择的提供商: {}", provider);
```

#### 基于模型名称的路由

```rust
// 系统会根据模型名称自动选择提供商
let model_providers = vec
![
    ("gpt-4o", "OpenAI"),
    ("deepseek-chat", "DeepSeek"),
    ("mixtral-8x7b-32768", "Groq"),
    ("glm-4.5", "GLM"),
];

for (model, expected_provider) in model_providers {
    let provider = router.select_provider(model, &config);
    println!("模型 {} -> 提供商 {}", model, provider);
}
```

## 错误处理

### 1. 基本错误处理

```rust
use orion_ai::{AiError, AiResult};

async fn handle_request() -> AiResult<()> {
    let client = AiClient::new(AiConfig::from_env()?, None)?;
    
    match client.send_request(request).await {
        Ok(response) => {
            println!("成功: {}", response.content);
            Ok(())
        }
        Err(AiError { reason, .. }) => {
            match reason {
                orion_ai::AiErrReason::RateLimitError(provider) => {
                    eprintln!("速率限制错误，提供商: {}", provider);
                }
                orion_ai::AiErrReason::TokenLimitError(requested, max) => {
                    eprintln!("Token限制: 请求 {}，最大 {}", requested, max);
                }
                orion_ai::AiErrReason::NoProviderAvailable => {
                    eprintln!("没有可用的提供商");
                }
                orion_ai::AiErrReason::InvalidModel(model) => {
                    eprintln!("无效的模型: {}", model);
                }
                _ => {
                    eprintln!("其他错误: {}", reason);
                }
            }
            Err(AiError::from(reason))
        }
    }
}
```

### 2. 重试机制

```rust
use std::time::Duration;
use tokio::time::sleep;

async fn retry_request<F, Fut>(request_func: F) -> AiResult<AiResponse>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = AiResult<AiResponse>>,
{
    let mut retries = 0;
    let max_retries = 3;
    
    loop {
        match request_func().await {
            Ok(response) => return Ok(response),
            Err(AiError { reason, .. }) => {
                match reason {
                    orion_ai::AiErrReason::RateLimitError(_) if retries < max_retries => {
                        retries += 1;
                        let delay = Duration::from_secs(2u64.pow(retries as u32));
                        println!("速率限制，等待 {:?} 后重试...", delay);
                        sleep(delay).await;
                        continue;
                    }
                    _ => return Err(AiError::from(reason)),
                }
            }
        }
    }
}
```

## 性能优化

### 1. 并发请求

```rust
use futures::future::join_all;

async fn concurrent_requests() -> AiResult<Vec<AiResponse>> {
    let client = AiClient::new(AiConfig::from_env()?, None)?;
    
    let requests = vec
![
        AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt("什么是Rust?")
            .build(),
        AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt("什么是异步编程?")
            .build(),
        AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt("什么是并发?")
            .build(),
    ];
    
    // 并发发送所有请求
    let futures = requests.into_iter().map(|req| client.send_request(req));
    let responses = join_all(futures).await;
    
    // 处理结果
    let mut successful_responses = Vec::new();
    for response in responses {
        match response {
            Ok(resp) => successful_responses.push(resp),
            Err(e) => eprintln!("请求失败: {}", e),
        }
    }
    
    Ok(successful_responses)
}
```

### 2. 客户端复用

```rust
use std::sync::Arc;

// 共享客户端
async fn shared_client_usage() -> AiResult<()> {
    let client = Arc::new(AiClient::new(AiConfig::from_env()?, None)?);
    
    // 在多个任务中使用同一个客户端
    let client1 = client.clone();
    let client2 = client.clone();
    
    let task1 = tokio::spawn(async move {
        let request = AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt("任务1")
            .build();
        client1.send_request(request).await
    });
    
    let task2 = tokio::spawn(async move {
        let request = AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt("任务2")
            .build();
        client2.send_request(request).await
    });
    
    let (result1, result2) = tokio::try_join!(task1, task2)?;
    
    println!("任务1结果: {:?}", result1?);
    println!("任务2结果: {:?}", result2?);
    
    Ok(())
}
```

## 测试

### 1. 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use orion_ai::provider::AiProviderType;

    #[test]
    fn test_role_creation() {
        let role = AiRoleID::new("test");
        assert_eq!(role.id(), "test");
        assert_eq!(role.description(), "ai-role: test");
    }

    #[tokio::test]
    async fn test_mock_provider() {
        let config = AiConfig::example();
        let client = AiClient::new(config, None).unwrap();
        
        let request = AiRequest::builder()
            .model("mock-gpt")
            .user_prompt("测试")
            .build();
        
        let response = client.send_request(request).await.unwrap();
        assert!(response.content.contains("MOCK"));
    }

    #[test]
    fn test_router_selection() {
        let router = AiRouter::new();
        let config = AiConfig::default();
        
        assert_eq!(
            router.select_provider("gpt-4o", &config),
            AiProviderType::OpenAi
        );
        assert_eq!(
            router.select_provider("deepseek-chat", &config),
            AiProviderType::DeepSeek
        );
    }
}
```

### 2. 集成测试

```rust
#[tokio::test]
async fn test_full_workflow() {
    // 创建配置
    let mut config = AiConfig::example();
    config.thread.enabled = true;
    
    // 创建客户端
    let client = AiClient::new(config, None).unwrap();
    
    // 创建角色
    let role = AiRoleID::new("test_role");
    
    // 发送请求
    let request = AiRequest::builder()
        .model("mock-gpt")
        .user_prompt("这是一个集成测试")
        .role(role.clone())
        .build();
    
    let response = client.send_request(request).await.unwrap();
    
    // 验证响应
    assert!(!response.content.is_empty());
    assert_eq!(response.model, "mock-gpt");
    assert!(response.usage.total_tokens > 0);
}
```

## 最佳实践

### 1. 配置管理

```rust
// 推荐的配置加载方式
pub fn create_client() -> AiResult<AiClient> {
    // 1. 尝试从项目配置文件加载
    let config = if std::path::Path::new("ai.yml").exists() {
        orion_ai::config::ConfigLoader::from_file("ai.yml")?
    } else {
        // 2. 回退到环境变量
        AiConfig::from_env()?
    };
    
    // 3. 检查角色配置文件
    let role_file = std::path::Path::new("ai-roles.yml")
        .exists()
        .then(|| std::path::PathBuf::from("ai-roles.yml"));
    
    // 4. 创建客户端
    AiClient::new(config, role_file)
}
```

### 2. 错误处理和重试

```rust
pub async fn safe_ai_request(
    client: &AiClient,
    request: AiRequest,
) -> Result<AiResponse, String> {
    const MAX_RETRIES: u32 = 3;
    const BASE_DELAY: u64 = 1;
    
    for attempt in 0..MAX_RETRIES {
        match client.send_request(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(AiError { reason, .. }) => {
                match reason {
                    orion_ai::AiErrReason::RateLimitError(provider) if attempt < MAX_RETRIES - 1 => {
                        let delay = BASE_DELAY * 2u64.pow(attempt);
                        tokio::time::sleep(tokio::time::Duration::from_secs(delay)).await;
                        continue;
                    }
                    _ => return Err(format!("AI请求失败: {}", reason)),
                }
            }
        }
    }
    
    Err("重试次数超限".to_string())
}
```

### 3. 资源管理

```rust
pub struct AiService {
    client: Arc<AiClient>,
    timeout: Duration,
}

impl AiService {
    pub fn new(client: AiClient) -> Self {
        Self {
            client: Arc::new(client),
            timeout: Duration::from_secs(30),
        }
    }
    
    pub async fn ask(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error>> {
        let request = AiRequest::builder()
            .model("gpt-4o-mini")
            .user_prompt(prompt.to_string())
            .build();
        
        let response = tokio::time::timeout(
            self.timeout,
            self.client.send_request(request)
        ).await??;
        
        Ok(response.content)
    }
}
```

## 总结

Orion AI 提供了强大且灵活的 AI 客户端功能：

1. **统一接口**: 通过统一的API与多个AI提供商交互
2. **角色系统**: 支持基于角色的智能请求处理
3. **线程记录**: 自动记录和管理对话历史
4. **灵活配置**: 支持多种配置来源和动态配置
5. **错误处理**: 完善的错误处理和重试机制
6. **性能优化**: 支持并发请求和客户端复用

通过遵循本文档中的最佳实践，您可以构建高效、可靠的 AI 应用程序。