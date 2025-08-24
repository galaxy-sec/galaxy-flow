# Orion AI 文档

## 概述

Orion AI 是一个用 Rust 编写的多提供商 AI 客户端库，提供了统一的接口来与不同的 AI 服务提供商进行交互。该库设计优雅，支持角色配置、线程记录、智能路由等高级功能。

## 文档结构

### 📚 核心文档

1. **[代码结构分析](./code_structure.md)**
   - 详细的项目结构说明
   - 核心模块分析
   - 设计模式解析
   - 依赖关系图
   - 扩展性设计

2. **[模块关系图](./module_relationships.md)**
   - 模块依赖关系图
   - 数据流向图
   - 接口关系层次
   - 生命周期关系
   - 扩展点分析

3. **[API使用文档](./api_usage.md)**
   - 基础使用指南
   - 高级功能详解
   - 配置管理
   - 错误处理
   - 性能优化
   - 测试指南
   - 最佳实践

## 快速开始

### 安装

在您的 `Cargo.toml` 中添加依赖：

```toml
[dependencies]
orion_ai = { path = "../crates/orion_ai" }
tokio = { version = "1.0", features = ["full"] }
```

### 基础使用

```rust
use orion_ai::{AiClient, AiConfig, AiRequest, AiRoleID};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 从环境变量加载配置
    let config = AiConfig::from_env()?;
    
    // 创建客户端
    let client = AiClient::new(config, None)?;
    
    // 构建请求
    let request = AiRequest::builder()
        .model("gpt-4o-mini")
        .user_prompt("请解释什么是Rust编程语言")
        .build();
    
    // 发送请求
    let response = client.send_request(request).await?;
    println!("AI响应: {}", response.content);
    
    Ok(())
}
```

### 角色驱动使用

```rust
// 创建角色
let role = AiRoleID::new("developer");

// 发送基于角色的请求
let response = client.smart_role_request(&role, "请帮我优化这段代码").await?;
println!("角色响应: {}", response.content);
```

## 核心特性

### 🎯 多提供商支持

- **OpenAI**: 完整支持所有OpenAI模型
- **DeepSeek**: 99.5%成本降低的替代方案
- **Groq**: 高速推理支持
- **Kimi**: 月之暗面大模型
- **GLM**: 智谱AI大模型
- **Mock**: 测试和开发用模拟提供商

### 🎭 角色系统

- **动态角色配置**: 支持运行时角色定义和加载
- **智能角色路由**: 根据角色自动选择合适的模型
- **规则增强**: 为角色添加特定规则和约束
- **分层配置**: 支持全局、项目、用户多层配置

### 🧵 线程记录

- **自动对话记录**: 自动保存所有对话历史
- **智能摘要**: 从长对话中提取关键信息
- **上下文保持**: 在连续对话中维护上下文
- **灵活存储**: 支持自定义存储路径和格式

### 🛣️ 智能路由

- **模型识别**: 根据模型名称自动选择提供商
- **成本优化**: 自动选择成本效益最高的方案
- **负载均衡**: 支持多提供商负载均衡
- **自定义规则**: 支持用户自定义路由规则

## 项目架构

### 设计原则

1. **分层架构**: 清晰的层次分离，每层有明确的职责
2. **接口隔离**: 通过 trait 定义清晰的接口边界
3. **依赖倒置**: 高层模块不依赖低层模块的实现细节
4. **开闭原则**: 对扩展开放，对修改关闭
5. **单一职责**: 每个模块都有明确的单一职责

### 核心模块

```
src/
├── lib.rs              # 库入口，模块导出
├── provider.rs         # 核心提供商定义和接口
├── error.rs           # 错误处理定义
├── client/            # 客户端实现
├── config/            # 配置管理
├── providers/         # 提供商实现
├── thread/            # 线程管理
├── factory.rs         # 工厂模式
└── router.rs          # 路由管理
```

### 设计模式

- **工厂模式**: 用于创建不同类型的客户端
- **策略模式**: 不同的提供商实现相同的接口
- **模板方法模式**: 定义客户端的标准操作流程
- **装饰器模式**: 为基础客户端添加额外功能
- **建造者模式**: 构建复杂的客户端配置

## 配置指南

### 环境变量

```bash
# OpenAI 配置
export OPENAI_API_KEY="your-openai-api-key"
export OPENAI_BASE_URL="https://api.openai.com/v1"

# DeepSeek 配置
export DEEPSEEK_API_KEY="your-deepseek-api-key"
export DEEPSEEK_BASE_URL="https://api.deepseek.com/v1"

# 其他提供商...
```

### 配置文件

```yaml
# ai.yml
providers:
  openai:
    enabled: true
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    priority: 1
  
  deepseek:
    enabled: true
    api_key: "${DEEPSEEK_API_KEY}"
    base_url: "https://api.deepseek.com/v1"
    priority: 2

thread:
  enabled: true
  storage_path: "./threads"
  min_summary_length: 200
  max_summary_length: 500
```

### 角色配置

```yaml
# ai-roles.yml
default_role:
  id: galactiward
default_model: deepseek-chat

roles:
  developer:
    name: developer
    description: 专注于代码开发的技术专家
    system_prompt: 你是一个专业的开发者，擅长高质量的代码实现。
    used_model: deepseek-coder
  
  operations:
    name: operations
    description: 专注于系统运维的专家
    system_prompt: 你是一个专业的运维专家，擅长诊断系统问题。
    used_model: gpt-4o
```

## 使用场景

### 1. 代码助手

```rust
let developer_role = AiRoleID::new("developer");
let response = client.smart_role_request(
    &developer_role, 
    "请帮我优化这段Rust代码的性能"
).await?;
```

### 2. 文档生成

```rust
let request = AiRequest::builder()
    .model("gpt-4o")
    .system_prompt("你是一个技术文档专家")
    .user_prompt("请为以下API生成使用文档")
    .build();
```

### 3. 代码审查

```rust
let reviewer_role = AiRoleID::new("code-reviewer");
let code = "fn main() { println!(\"Hello\"); }";
let response = client.smart_role_request(
    &reviewer_role,
    &format!("请审查这段代码：\n{}", code)
).await?;
```

### 4. 系统运维

```rust
let ops_role = AiRoleID::new("operations");
let log_content = "ERROR: Connection failed";
let response = client.smart_role_request(
    &ops_role,
    &format!("请分析这个错误日志：\n{}", log_content)
).await?;
```

## 扩展开发

### 添加新提供商

1. 实现 `AiProvider` trait
2. 在 `AiProviderType` 中添加新类型
3. 更新路由逻辑
4. 在工厂中注册新提供商

### 自定义角色处理

1. 扩展 `RoleConfig` 结构
2. 实现自定义的角色加载逻辑
3. 添加角色特定的处理规则
4. 集成到客户端中

### 添加中间件

1. 创建装饰器客户端
2. 实现请求/响应拦截
3. 添加缓存、限流等功能
4. 集成到客户端构建过程

## 性能优化

### 并发请求

```rust
use futures::future::join_all;

let requests = vec![request1, request2, request3];
let responses = join_all(
    requests.into_iter().map(|req| client.send_request(req))
).await;
```

### 客户端复用

```rust
use std::sync::Arc;
let client = Arc::new(AiClient::new(config, None)?);
// 在多个任务中共享客户端
```

### 连接池配置

```rust
// 通过配置文件设置连接池
providers:
  openai:
    timeout: 30
    max_connections: 10
```

## 错误处理

### 错误类型

- `RateLimitError`: API速率限制
- `TokenLimitError`: Token数量限制
- `ContextError`: 上下文收集失败
- `NoProviderAvailable`: 无可用提供商
- `InvalidModel`: 无效的模型名称
- `SensitiveContentFiltered`: 敏感内容被过滤

### 重试机制

```rust
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
                        tokio::time::sleep(Duration::from_secs(2u64.pow(retries))).await;
                        continue;
                    }
                    _ => return Err(AiError::from(reason)),
                }
            }
        }
    }
}
```

## 贡献指南

### 开发环境设置

1. 克隆仓库
2. 安装 Rust 工具链
3. 配置环境变量
4. 运行测试：`cargo test`
5. 格式化代码：`cargo fmt`
6. 检查代码：`cargo clippy`

### 代码规范

- 遵循 Rust 官方代码风格
- 编写完整的文档注释
- 包含单元测试和集成测试
- 使用错误处理最佳实践
- 保持模块职责单一

### 提交规范

- feat: 新功能
- fix: 错误修复
- docs: 文档更新
- style: 代码格式调整
- refactor: 代码重构
- test: 测试相关
- chore: 构建或工具变动

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE](../../LICENSE) 文件。

## 支持

如果您在使用过程中遇到问题，请：

1. 查看本文档的 FAQ 部分
2. 检查 [Issues](../../issues) 是否有类似问题
3. 创建新的 Issue 描述您的问题
4. 参与 [Discussions](../../discussions) 社区讨论

## 更新日志

详见 [CHANGELOG.md](../../CHANGELOG.md) 文件。

---

**注意**: 本文档会随着项目的发展持续更新。建议定期查看最新版本以获取最新的功能和使用说明。