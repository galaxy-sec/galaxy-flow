# Galaxy-flow AI 模块重构方案

## 项目概述

本文档详细描述了对 `src/ability/ai` 模块的重构方案。当前AI模块存在代码组织混乱、职责不清、错误处理不一致等问题。通过系统性的重构，我们将建立一个可维护、可扩展、高性能的AI功能架构。

## 重构目标

### 核心目标
- **架构清晰化**：建立清晰的模块职责边界
- **代码标准化**：统一编码规范和错误处理
- **性能优化**：提升AI功能的执行效率
- **可扩展性**：支持第三方工具和AI提供商扩展

### 技术目标
- **模块化设计**：各组件职责单一，低耦合高内聚
- **接口标准化**：建立统一的trait接口体系
- **错误处理统一**：使用 `thiserror` 统一错误类型
- **测试覆盖完善**：单元测试和集成测试覆盖率 > 80%

## 当前问题分析

### 1. 架构问题

#### 问题1：职责边界模糊
```rust
// 当前问题：ai_fun.rs 承担过多职责
pub struct GxAIFun {
    role: Option<String>,
    task: Option<String>,
    config: Option<AiConfig>,
    tools: Vec<String>,
    max_rounds: usize,
}

// 问题：execute_with_retry_and_storage 函数过长且职责混乱
async fn execute_with_retry_and_storage(&self, mut ctx: ExecContext, mut vars_dict: VarSpace) -> TaskResult {
    // 混合了：上下文管理、AI调用、重试逻辑、结果存储
    ctx.append("gx.ai_fun");
    let mut action = Action::from("gx.ai_fun");
    // ... 50+ 行代码混合多种职责
}
```

#### 问题2：模块组织混乱
```
src/ability/ai/
├── ai_chat.rs      # 简单聊天功能，但命名不清晰
├── ai_fun.rs       # 函数调用功能，承担过多职责
├── tool.rs         # 工具相关，但功能不完整
└── mod.rs          # 模块导出
```

### 2. 代码质量问题

#### 问题3：错误处理不一致
```rust
// ai_chat.rs 中的错误处理
let ai_config = self
    .config()
    .clone()
    .unwrap_or(AiConfig::galaxy_load(&vars_dict.global().export().into()).err_conv()?);

// ai_fun.rs 中的错误处理
let ai_config = AiConfig::galaxy_load(&vars_dict.global().export().into())
    .map_err(|e| ExecReason::from_conf(format!("加载AI配置失败: {}", e)))?;
```

#### 问题4：硬编码和魔法数字
```rust
// ai_fun.rs
max_rounds: usize, // 最大重试轮次，默认为3
impl Default for GxAIFun {
    fn default() -> Self {
        Self {
            // ...
            max_rounds: 3, // 🎯 默认最大重试3次
        }
    }
}
```

### 3. 测试质量问题

#### 问题5：测试依赖外部服务
```rust
#[tokio::test]
async fn test_basic_ai_chat() {
    let config = if let Some(dict) = load_key_dict("sec_deepseek_api_key") {
        AiConfig::example().env_eval(&dict)
    } else {
        return; // 直接返回，测试无法验证
    };
    // ...
}
```

## 重构方案设计

### 1. 新的模块架构

#### 目标架构
```
src/ability/ai/
├── mod.rs                      # 模块导出和文档
├── constants.rs                # 常量定义
├── errors.rs                   # 统一错误处理
├── client/                     # AI客户端相关
│   ├── mod.rs
│   ├── executor.rs            # 执行器trait和基础实现
│   ├── simple_chat.rs         # 简单聊天功能
│   ├── function_calling.rs    # 函数调用功能
│   └── config.rs              # 客户端配置
├── tools/                      # 工具管理
│   ├── mod.rs
│   ├── registry.rs            # 工具注册表
│   ├── executor.rs            # 工具执行器trait
│   └── builtin.rs             # 内置工具实现
├── session/                    # 会话管理
│   ├── mod.rs
│   ├── manager.rs             # 会话管理器
│   ├── state.rs               # 会话状态
│   └── result.rs              # 执行结果
└── config/                     # 配置管理
    ├── mod.rs
    ├── validator.rs           # 配置验证
    └── loader.rs              # 配置加载
```

#### 架构优势
- **职责分离**：每个模块职责单一明确
- **低耦合**：通过trait接口实现松耦合
- **高内聚**：相关功能集中在对应模块
- **可扩展**：支持插件化扩展

### 2. 核心结构体重构

#### 2.1 FunctionCallingExecutor (重构自 GxAIFun)

**新结构设计：**
```rust
// client/function_calling.rs
use std::sync::Arc;
use tokio::sync::Semaphore;

#[derive(Clone, Debug)]
pub struct FunctionCallingExecutor {
    config: FunctionCallingConfig,
    client: Arc<dyn AiClientTrait>,
    tool_registry: Arc<ToolRegistry>,
    session_manager: Arc<SessionManager>,
    concurrency_limiter: Arc<Semaphore>,
}

impl FunctionCallingExecutor {
    pub fn new(
        config: FunctionCallingConfig,
        client: Arc<dyn AiClientTrait>,
        tool_registry: Arc<ToolRegistry>,
    ) -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new(config.session_config.clone())),
            concurrency_limiter: Arc::new(Semaphore::new(config.max_concurrent_tasks)),
            config,
            client,
            tool_registry,
        }
    }

    pub async fn execute(&self, task: &TaskDescription) -> Result<ExecutionResult, AiError> {
        // 获取并发许可
        let _permit = self.concurrency_limiter.acquire().await.unwrap();
        
        // 创建会话
        let session_id = self.session_manager.create_session().await?;
        
        // 执行任务
        let result = self.execute_with_retry(session_id, task).await;
        
        // 清理会话
        self.session_manager.cleanup_session(&session_id).await;
        
        result
    }

    async fn execute_with_retry(
        &self,
        session_id: String,
        task: &TaskDescription,
    ) -> Result<ExecutionResult, AiError> {
        let mut retry_count = 0;
        let max_retries = self.config.retry_policy.max_rounds;
        
        loop {
            match self.execute_single_round(&session_id, task, retry_count).await {
                Ok(result) => {
                    if result.is_success() {
                        return Ok(result);
                    }
                    
                    retry_count += 1;
                    if retry_count >= max_retries {
                        return Err(AiError::RetryExceeded { 
                            max_rounds: max_retries,
                            last_error: result.error_message().clone(),
                        });
                    }
                    
                    // 应用退避策略
                    tokio::time::sleep(
                        self.config.retry_policy.calculate_delay(retry_count)
                    ).await;
                }
                Err(e) => return Err(e),
            }
        }
    }

    async fn execute_single_round(
        &self,
        session_id: &str,
        task: &TaskDescription,
        round: usize,
    ) -> Result<ExecutionResult, AiError> {
        // 构建提示词
        let prompt = self.build_prompt(task, session_id, round).await?;
        
        // 发送AI请求
        let ai_response = self.client
            .request_with_tools(&prompt, self.tool_registry.get_tool_definitions())
            .await?;
        
        // 执行工具调用
        self.execute_tool_calls(ai_response, session_id).await
    }

    async fn execute_tool_calls(
        &self,
        ai_response: AiResponse,
        session_id: &str,
    ) -> Result<ExecutionResult, AiError> {
        let mut tool_results = Vec::new();
        
        if let Some(calls) = ai_response.tool_calls {
            for tool_call in calls {
                let result = self.execute_single_tool_call(tool_call, session_id).await;
                tool_results.push(result);
            }
        }
        
        Ok(ExecutionResult::new(
            session_id.to_string(),
            tool_results,
            ai_response.content,
            ai_response.metadata,
        ))
    }

    async fn execute_single_tool_call(
        &self,
        tool_call: ToolCall,
        session_id: &str,
    ) -> ToolCallResult {
        match self.tool_registry.execute_tool(&tool_call).await {
            Ok(output) => ToolCallResult::success(
                tool_call.function.name,
                output,
                std::time::Instant::now().elapsed(),
            ),
            Err(e) => ToolCallResult::failure(
                tool_call.function.name,
                e.to_string(),
                std::time::Instant::now().elapsed(),
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct FunctionCallingConfig {
    pub role: Option<String>,
    pub base_prompt: String,
    pub retry_policy: RetryPolicy,
    pub session_config: SessionConfig,
    pub max_concurrent_tasks: usize,
    pub enable_tool_validation: bool,
}

#[derive(Clone, Debug)]
pub struct RetryPolicy {
    pub max_rounds: usize,
    pub base_delay_ms: u64,
    pub backoff_multiplier: f64,
    pub max_delay_ms: u64,
}

impl RetryPolicy {
    pub fn calculate_delay(&self, retry_count: usize) -> Duration {
        let delay_ms = (self.base_delay_ms as f64 * 
                        self.backoff_multiplier.powi(retry_count as i32 - 1))
                        .min(self.max_delay_ms as f64) as u64;
        Duration::from_millis(delay_ms)
    }
}
```

**重构价值分析：**
- ✅ **职责分离**：将原来的单个大函数拆分为多个小函数
- ✅ **并发控制**：添加信号量限制并发任务数
- ✅ **智能重试**：实现退避策略和智能重试机制
- ✅ **资源管理**：完善的会话生命周期管理
- ✅ **错误处理**：统一的错误类型和处理机制

#### 2.2 ToolRegistry (新结构体)

**新结构设计：**
```rust
// tools/registry.rs
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct ToolRegistry {
    tools: Arc<RwLock<HashMap<String, Box<dyn ToolExecutor>>>>,
    permissions: Arc<RwLock<HashMap<String, PermissionLevel>>>,
    metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    config: RegistryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMetadata {
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub tags: Vec<String>,
    pub schema: ToolSchema,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum PermissionLevel {
    ReadOnly,
    Write,
    Admin,
}

impl ToolRegistry {
    pub fn new(config: RegistryConfig) -> Self {
        let registry = Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            config,
        };

        // 注册内置工具
        registry.register_builtin_tools();
        registry
    }

    pub fn register_tool<T>(&self, name: String, tool: T, permission: PermissionLevel) -> Result<(), ToolError>
    where
        T: ToolExecutor + 'static,
    {
        let metadata = ToolMetadata {
            name: name.clone(),
            description: tool.description().to_string(),
            version: tool.version().to_string(),
            author: tool.author().to_string(),
            tags: tool.tags().to_vec(),
            schema: tool.schema(),
            dependencies: tool.dependencies(),
        };

        // 验证工具
        self.validate_tool(&metadata, &tool)?;

        // 注册工具
        {
            let mut tools = self.tools.write().unwrap();
            let mut permissions = self.permissions.write().unwrap();
            let mut metadata_map = self.metadata.write().unwrap();

            tools.insert(name.clone(), Box::new(tool));
            permissions.insert(name.clone(), permission);
            metadata_map.insert(name, metadata);
        }

        Ok(())
    }

    pub async fn execute_tool(&self, tool_call: &ToolCall) -> Result<ToolOutput, ToolError> {
        // 检查权限
        self.check_permission(&tool_call.function.name)?;

        // 获取工具
        let tool = {
            let tools = self.tools.read().unwrap();
            tools.get(&tool_call.function.name)
                .ok_or_else(|| ToolError::ToolNotFound(tool_call.function.name.clone()))?
                .clone()
        };

        // 验证参数
        tool.validate_params(&tool_call.function.arguments)?;

        // 执行工具
        let start_time = std::time::Instant::now();
        let result = tool.execute(&tool_call.function.arguments).await;
        let execution_time = start_time.elapsed();

        match result {
            Ok(output) => {
                // 记录执行统计
                self.record_execution_stats(&tool_call.function.name, execution_time, true);
                Ok(output)
            }
            Err(e) => {
                // 记录失败统计
                self.record_execution_stats(&tool_call.function.name, execution_time, false);
                Err(e)
            }
        }
    }

    pub fn get_tool_definitions(&self) -> Vec<ToolDefinition> {
        let tools = self.tools.read().unwrap();
        let metadata = self.metadata.read().unwrap();

        tools.iter()
            .filter_map(|(name, tool)| {
                metadata.get(name).map(|meta| ToolDefinition {
                    name: name.clone(),
                    description: meta.description.clone(),
                    parameters: tool.parameters(),
                })
            })
            .collect()
    }

    pub fn get_supported_function_names(&self) -> Vec<String> {
        let tools = self.tools.read().unwrap();
        tools.keys().cloned().collect()
    }

    fn validate_tool(&self, metadata: &ToolMetadata, tool: &dyn ToolExecutor) -> Result<(), ToolError> {
        // 验证工具名称
        if metadata.name.is_empty() {
            return Err(ToolError::InvalidToolName("Tool name cannot be empty".into()));
        }

        // 验证工具描述
        if metadata.description.len() < 10 {
            return Err(ToolError::InvalidDescription("Description too short".into()));
        }

        // 验证依赖关系
        for dep in &metadata.dependencies {
            let tools = self.tools.read().unwrap();
            if !tools.contains_key(dep) {
                return Err(ToolError::DependencyNotFound(dep.clone()));
            }
        }

        // 验证工具自身
        tool.validate()?;

        Ok(())
    }

    fn check_permission(&self, tool_name: &str) -> Result<(), ToolError> {
        let permissions = self.permissions.read().unwrap();
        let permission = permissions.get(tool_name)
            .ok_or_else(|| ToolError::PermissionDenied(tool_name.to_string()))?;

        // 这里可以添加更复杂的权限检查逻辑
        match permission {
            PermissionLevel::ReadOnly => Ok(()),
            PermissionLevel::Write => Ok(()),
            PermissionLevel::Admin => Ok(()),
        }
    }

    fn record_execution_stats(&self, tool_name: &str, execution_time: Duration, success: bool) {
        // 记录执行统计信息，用于监控和优化
        // 这里可以集成到监控系统中
        if self.config.enable_stats {
            log::info!(
                "Tool '{}' executed in {}ms, success: {}",
                tool_name,
                execution_time.as_millis(),
                success
            );
        }
    }

    fn register_builtin_tools(&self) {
        // 注册内置的 Git 工具
        let git_tools = builtin::create_git_tools();
        for (name, tool) in git_tools {
            let _ = self.register_tool(
                name,
                tool,
                PermissionLevel::Write, // Git 工具需要写权限
            );
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegistryConfig {
    pub enable_stats: bool,
    pub max_tools: usize,
    pub enable_validation: bool,
    pub builtin_tools_enabled: bool,
}

impl Default for RegistryConfig {
    fn default() -> Self {
        Self {
            enable_stats: true,
            max_tools: 1000,
            enable_validation: true,
            builtin_tools_enabled: true,
        }
    }
}
```

**重构价值分析：**
- ✅ **工具管理**：统一的工具注册、发现和执行机制
- ✅ **权限控制**：细粒度的工具权限管理
- ✅ **元数据管理**：完整的工具元数据支持
- ✅ **依赖管理**：工具间依赖关系的验证和管理
- ✅ **统计监控**：工具执行统计和性能监控

#### 2.3 ExecutionResult (重构自 ToolCallResult)

**新结构设计：**
```rust
// session/result.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    pub session_id: String,
    pub tool_calls: Vec<ToolCallResult>,
    pub final_response: Option<String>,
    pub metadata: ExecutionMetadata,
    pub error_context: Option<ErrorContext>,
    pub execution_summary: ExecutionSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResult {
    pub tool_name: String,
    pub execution_id: String,
    pub status: ExecutionStatus,
    pub output: ToolOutput,
    pub execution_time: Duration,
    pub timestamp: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Success,
    Failure { error: String, error_type: String },
    Timeout,
    Cancelled,
}

impl ToolCallResult {
    pub fn success<S: Into<String>>(
        tool_name: S,
        output: ToolOutput,
        execution_time: Duration,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            status: ExecutionStatus::Success,
            output,
            execution_time,
            timestamp: Utc::now(),
            resource_usage: ResourceUsage::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn failure<S: Into<String>, E: Into<String>>(
        tool_name: S,
        error: E,
        error_type: S,
        execution_time: Duration,
    ) -> Self {
        Self {
            tool_name: tool_name.into(),
            execution_id: uuid::Uuid::new_v4().to_string(),
            status: ExecutionStatus::Failure {
                error: error.into(),
                error_type: error_type.into(),
            },
            output: ToolOutput::Text("".to_string()),
            execution_time,
            timestamp: Utc::now(),
            resource_usage: ResourceUsage::default(),
            metadata: HashMap::new(),
        }
    }

    pub fn is_success(&self) -> bool {
        matches!(self.status, ExecutionStatus::Success)
    }

    pub fn error_message(&self) -> Option<&str> {
        match &self.status {
            ExecutionStatus::Failure { error, .. } => Some(error),
            _ => None,
        }
    }

    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    pub fn with_resource_usage(mut self, usage: ResourceUsage) -> Self {
        self.resource_usage = usage;
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_duration: Duration,
    pub ai_provider: String,
    pub ai_model: String,
    pub total_tokens: Option<u32>,
    pub cost_estimate: Option<f64>,
    pub retry_count: usize,
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_chain: Vec<ErrorInfo>,
    pub recovery_attempts: Vec<RecoveryAttempt>,
    pub context_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionSummary {
    pub total_tool_calls: usize,
    pub successful_calls: usize,
    pub failed_calls: usize,
    pub total_execution_time: Duration,
    pub average_execution_time: Duration,
    pub success_rate: f64,
}

impl ExecutionResult {
    pub fn new(
        session_id: String,
        tool_calls: Vec<ToolCallResult>,
        final_response: Option<String>,
        metadata: ExecutionMetadata,
    ) -> Self {
        let summary = Self::calculate_summary(&tool_calls);
        
        Self {
            session_id,
            tool_calls,
            final_response,
            metadata,
            error_context: None,
            execution_summary: summary,
        }
    }

    pub fn with_error_context(mut self, error_context: ErrorContext) -> Self {
        self.error_context = Some(error_context);
        self
    }

    pub fn is_success(&self) -> bool {
        self.tool_calls.iter().all(|call| call.is_success())
    }

    pub fn failed_calls(&self) -> Vec<&ToolCallResult> {
        self.tool_calls
            .iter()
            .filter(|call| !call.is_success())
            .collect()
    }

    pub fn get_tool_result(&self, tool_name: &str) -> Option<&ToolCallResult> {
        self.tool_calls
            .iter()
            .find(|call| call.tool_name == tool_name)
    }

    pub fn total_cost(&self) -> Option<f64> {
        self.metadata.cost_estimate
    }

    fn calculate_summary(tool_calls: &[ToolCallResult]) -> ExecutionSummary {
        let total_tool_calls = tool_calls.len();
        let successful_calls = tool_calls.iter().filter(|call| call.is_success()).count();
        let failed_calls = total_tool_calls - successful_calls;
        
        let total_execution_time: Duration = tool_calls.iter()
            .map(|call| call.execution_time)
            .sum();
        
        let average_execution_time = if total_tool_calls > 0 {
            total_execution_time / total_tool_calls as u32
        } else {
            Duration::from_secs(0)
        };
        
        let success_rate = if total_tool_calls > 0 {
            successful_calls as f64 / total_tool_calls as f64
        } else {
            0.0
        };
        
        ExecutionSummary {
            total_tool_calls,
            successful_calls,
            failed_calls,
            total_execution_time,
            average_execution_time,
            success_rate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub memory_mb: f64,
    pub cpu_seconds: f64,
    pub network_bytes: u64,
    pub disk_io_bytes: u64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            memory_mb: 0.0,
            cpu_seconds: 0.0,
            network_bytes: 0,
            disk_io_bytes: 0,
        }
    }
}
```

**重构价值分析：**
- ✅ **结果结构化**：丰富的执行结果和元数据
- ✅ **错误追踪**：完整的错误上下文和恢复机制
- ✅ **性能监控**：资源使用和执行时间统计
- ✅ **会话管理**：支持会话级别的结果聚合
- ✅ **序列化支持**：便于存储和传输

### 3. 核心Trait重构

#### 3.1 AiExecutor Trait

**新Trait设计：**
```rust
// client/executor.rs
use std::fmt::Debug;
use async_trait::async_trait;

#[async_trait]
pub trait AiExecutor: Send + Sync + Debug {
    type Config: Clone + Debug + Send + Sync;
    type Input: Send + Sync;
    type Output: Send + Sync;

    /// 执行AI任务
    async fn execute(&self, input: &Self::Input) -> Result<Self::Output, AiError>;
    
    /// 验证配置
    fn validate_config(&self, config: &Self::Config) -> Result<(), ConfigError>;
    
    /// 获取执行器能力描述
    fn capabilities(&self) -> Vec<ExecutorCapability>;
    
    /// 获取支持的AI功能
    fn supported_features(&self) -> Vec<AiFeature>;
    
    /// 获取执行器状态
    fn status(&self) -> ExecutorStatus;
    
    /// 重置执行器状态
    async fn reset(&self) -> Result<(), AiError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutorCapability {
    pub name: String,
    pub description: String,
    pub version: String,
    pub dependencies: Vec<String>,
    pub configuration_schema: serde_json::Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AiFeature {
    Chat,
    ToolCalling,
    FunctionExecution,
    CodeGeneration,
    TextAnalysis,
    ImageProcessing,
    DataProcessing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutorStatus {
    Ready,
    Initializing,
    Busy,
    Error(String),
    Maintenance,
}

#[async_trait]
impl<T> AiExecutor for Arc<T>
where
    T: AiExecutor + ?Sized,
{
    type Config = T::Config;
    type Input = T::Input;
    type Output = T::Output;

    async fn execute(&self, input: &Self::Input) -> Result<Self::Output, AiError> {
        self.as_ref().execute(input).await
    }
    
    fn validate_config(&self, config: &Self::Config) -> Result<(), ConfigError> {
        self.as_ref().validate_config(config)
    }
    
    fn capabilities(&self) -> Vec<ExecutorCapability> {
        self.as_ref().capabilities()
    }
    
    fn supported_features(&self) -> Vec<AiFeature> {
        self.as_ref().supported_features()
    }
    
    fn status(&self) -> ExecutorStatus {
        self.as_ref().status()
    }
    
    async fn reset(&self) -> Result<(), AiError> {
        self.as_ref().reset().await
    }
}
```

#### 3.2 ToolExecutor Trait

**新Trait设计：**
```rust
// tools/executor.rs
use serde_json::Value;
use async_trait::async_trait;

#[async_trait]
pub trait ToolExecutor: Send + Sync + Debug {
    /// 工具名称
    fn name(&self) -> &str;
    
    /// 工具描述
    fn description(&self) -> &str;
    
    /// 工具版本
    fn version(&self) -> &str;
    
    /// 工具作者
    fn author(&self) -> &str;
    
    /// 工具标签
    fn tags(&self) -> &[String];
    
    /// 工具参数模式
    fn parameters(&self) -> ToolParameters;
    
    /// 执行工具
    async fn execute(&self, params: &ToolParameters) -> Result<ToolOutput, ToolError>;
    
    /// 验证参数
    fn validate_params(&self, params: &ToolParameters) -> Result<(), ToolError>;
    
    /// 工具模式
    fn schema(&self) -> ToolSchema;
    
    /// 工具依赖
    fn dependencies(&self) -> Vec<String>;
    
    /// 验证工具配置
    fn validate(&self) -> Result<(), ToolError>;
    
    /// 工具初始化
    async fn initialize(&self) -> Result<(), ToolError>;
    
    /// 工具清理
    async fn cleanup(&self) -> Result<(), ToolError>;
    
    /// 获取工具状态
    fn status(&self) -> ToolStatus;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolParameters {
    pub parameters: HashMap<String, Value>,
    pub context: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolOutput {
    Text(String),
    Json(Value),
    Binary(Vec<u8>),
    Stream(Box<dyn Stream<Item = Vec<u8>> + Send + Unpin>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub input_schema: Value,
    pub output_schema: Value,
    pub examples: Vec<ToolExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolExample {
    pub name: String,
    pub description: String,
    pub input: Value,
    pub output: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolStatus {
    Ready,
    Initializing,
    Busy,
    Error(String),
    Offline,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum ToolError {
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
    
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Tool not initialized: {0}")]
    NotInitialized(String),
    
    #[error("Timeout: {0}ms")]
    Timeout(u64),
    
    #[error("Invalid tool name: {0}")]
    InvalidToolName(String),
    
    #[error("Invalid description: {0}")]
    InvalidDescription(String),
    
    #[error("Dependency not found: {0}")]
    DependencyNotFound(String),
    
    #[error("Validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Other error: {0}")]
    Other(String),
}
```

### 4. 错误处理统一化

#### 4.1 统一错误类型

```rust
// errors.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AiError {
    #[error("配置错误: {0}")]
    ConfigError(#[from] ConfigError),
    
    #[error("客户端错误: {0}")]
    ClientError(#[from] ClientError),
    
    #[error("工具执行错误: {0}")]
    ToolError(#[from] ToolError),
    
    #[error("会话管理错误: {0}")]
    SessionError(#[from] SessionError),
    
    #[error("重试次数超限: 最大{max_rounds}次, 最后错误: {last_error}")]
    RetryExceeded { max_rounds: usize, last_error: String },
    
    #[error("网络错误: {0}")]
    NetworkError(String),
    
    #[error("权限错误: {0}")]
    PermissionError(String),
    
    #[error("验证错误: {0}")]
    ValidationError(String),
    
    #[error("内部错误: {0}")]
    InternalError(String),
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("配置加载失败: {0}")]
    LoadFailed(String),
    
    #[error("配置验证失败: {0}")]
    ValidationFailed(String),
    
    #[error("缺少必需配置: {0}")]
    MissingRequired(String),
    
    #[error("配置格式错误: {0}")]
    FormatError(String),
    
    #[error("配置值无效: {field} = {value}")]
    InvalidValue { field: String, value: String },
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("AI请求失败: {0}")]
    RequestFailed(String),
    
    #[error("响应解析失败: {0}")]
    ResponseParseFailed(String),
    
    #[error("API配额超限: {0}")]
    QuotaExceeded(String),
    
    #[error("认证失败: {0}")]
    AuthenticationFailed(String),
    
    #[error("服务不可用: {0}")]
    ServiceUnavailable(String),
    
    #[error("超时: {0}ms")]
    Timeout(u64),
}

pub type AiResult<T> = Result<T, AiError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type ClientResult<T> = Result<T, ClientError>;
```

### 5. 测试体系重构

#### 5.1 测试工具模块

```rust
// tests/test_utils.rs
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct MockAiClient {
    responses: Arc<RwLock<Vec<String>>>,
    should_fail: bool,
    delay_ms: u64,
}

impl MockAiClient {
    pub fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(RwLock::new(responses)),
            should_fail: false,
            delay_ms: 0,
        }
    }

    pub fn with_failure(mut self, should_fail: bool) -> Self {
        self.should_fail = should_fail;
        self
    }

    pub fn with_delay(mut self, delay_ms: u64) -> Self {
        self.delay_ms = delay_ms;
        self
    }

    pub async fn get_response_count(&self) -> usize {
        self.responses.read().await.len()
    }
}

#[async_trait]
impl AiClientTrait for MockAiClient {
    async fn request_with_tools(
        &self,
        prompt: &str,
        tools: Vec<ToolDefinition>,
    ) -> Result<AiResponse, AiError> {
        if self.delay_ms > 0 {
            tokio::time::sleep(Duration::from_millis(self.delay_ms)).await;
        }

        if self.should_fail {
            return Err(AiError::ClientError(ClientError::RequestFailed(
                "Mock failure".to_string(),
            )));
        }

        let response = {
            let mut responses = self.responses.write().await;
            responses.remove(0)
        };

        Ok(AiResponse {
            content: response,
            tool_calls: None,
            metadata: HashMap::new(),
        })
    }
}

pub struct TestConfigBuilder {
    config: FunctionCallingConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: FunctionCallingConfig {
                role: Some("test_role".to_string()),
                base_prompt: "Test prompt".to_string(),
                retry_policy: RetryPolicy {
                    max_rounds: 3,
                    base_delay_ms: 100,
                    backoff_multiplier: 2.0,
                    max_delay_ms: 5000,
                },
                session_config: SessionConfig::default(),
                max_concurrent_tasks: 1,
                enable_tool_validation: true,
            },
        }
    }

    pub fn with_role(mut self, role: String) -> Self {
        self.config.role = Some(role);
        self
    }

    pub fn with_max_rounds(mut self, max_rounds: usize) -> Self {
        self.config.retry_policy.max_rounds = max_rounds;
        self
    }

    pub fn build(self) -> FunctionCallingConfig {
        self.config
    }
}

pub fn create_test_registry() -> ToolRegistry {
    let config = RegistryConfig {
        enable_stats: false,
        max_tools: 100,
        enable_validation: true,
        builtin_tools_enabled: false, // 禁用内置工具以便测试
    };

    ToolRegistry::new(config)
}

pub fn create_mock_tool(name: String, should_fail: bool) -> MockTool {
    MockTool {
        name: name.clone(),
        description: format!("Mock tool {}", name),
        should_fail,
        execution_count: Arc::new(AtomicUsize::new(0)),
    }
}

pub struct MockTool {
    name: String,
    description: String,
    should_fail: bool,
    execution_count: Arc<AtomicUsize>,
}

#[async_trait]
impl ToolExecutor for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn author(&self) -> &str {
        "Test Author"
    }

    fn tags(&self) -> &[String] {
        &["test".to_string()]
    }

    fn parameters(&self) -> ToolParameters {
        ToolParameters {
            parameters: HashMap::new(),
            context: HashMap::new(),
        }
    }

    async fn execute(&self, _params: &ToolParameters) -> Result<ToolOutput, ToolError> {
        self.execution_count.fetch_add(1, Ordering::SeqCst);
        
        if self.should_fail {
            Err(ToolError::ExecutionFailed("Mock tool failure".to_string()))
        } else {
            Ok(ToolOutput::Text(format!("Mock tool {} executed successfully", self.name)))
        }
    }

    fn validate_params(&self, _params: &ToolParameters) -> Result<(), ToolError> {
        Ok(())
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema {
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            examples: vec![],
        }
    }

    fn dependencies(&self) -> Vec<String> {
        vec![]
    }

    fn validate(&self) -> Result<(), ToolError> {
        Ok(())
    }

    async fn initialize(&self) -> Result<(), ToolError> {
        Ok(())
    }

    async fn cleanup(&self) -> Result<(), ToolError> {
        Ok(())
    }

    fn status(&self) -> ToolStatus {
        ToolStatus::Ready
    }
}
```

#### 5.2 单元测试示例

```rust
// tests/unit/function_calling_test.rs
use super::test_utils::*;
use crate::client::function_calling::FunctionCallingExecutor;

#[tokio::test]
async fn test_successful_execution() {
    // Given
    let config = TestConfigBuilder::new().build();
    let mock_client = Arc::new(MockAiClient::new(vec![
        "Use tool test-tool".to_string(),
    ]));
    let registry = Arc::new(create_test_registry());
    
    registry
        .register_tool(
            "test-tool".to_string(),
            create_mock_tool("test-tool".to_string(), false),
            PermissionLevel::Write,
        )
        .unwrap();

    let executor = FunctionCallingExecutor::new(config, mock_client, registry);

    // When
    let task = TaskDescription::new("Execute test task");
    let result = executor.execute(&task).await;

    // Then
    assert!(result.is_ok());
    let exec_result = result.unwrap();
    assert!(exec_result.is_success());
    assert_eq!(exec_result.tool_calls.len(), 1);
    assert!(exec_result.tool_calls[0].is_success());
}

#[tokio::test]
async fn test_retry_mechanism() {
    // Given
    let config = TestConfigBuilder::new()
        .with_max_rounds(2)
        .build();
    let mock_client = Arc::new(MockAiClient::new(vec![
        "Use tool failing-tool".to_string(),
        "Use tool failing-tool".to_string(),
    ]).with_failure(true));
    let registry = Arc::new(create_test_registry());
    
    registry
        .register_tool(
            "failing-tool".to_string(),
            create_mock_tool("failing-tool".to_string(), true),
            PermissionLevel::Write,
        )
        .unwrap();

    let executor = FunctionCallingExecutor::new(config, mock_client, registry);

    // When
    let task = TaskDescription::new("Execute failing task");
    let result = executor.execute(&task).await;

    // Then
    assert!(result.is_err());
    match result.unwrap_err() {
        AiError::RetryExceeded { max_rounds, .. } => {
            assert_eq!(max_rounds, 2);
        }
        _ => panic!("Expected RetryExceeded error"),
    }
}

#[tokio::test]
async fn test_concurrency_limit() {
    // Given
    let config = TestConfigBuilder::new()
        .with_max_rounds(1)
        .build();
    let mock_client = Arc::new(MockAiClient::new(vec!["Response".to_string()]));
    let registry = Arc::new(create_test_registry());
    
    // 创建慢速工具
    registry
        .register_tool(
            "slow-tool".to_string(),
            create_mock_tool("slow-tool".to_string(), false),
            PermissionLevel::Write,
        )
        .unwrap();

    // 创建执行器，限制并发为1
    let executor = FunctionCallingExecutor::new(config, mock_client, registry);

    // When - 同时启动多个任务
    let task = TaskDescription::new("Concurrent task");
    let handles: Vec<_> = (0..3)
        .map(|_| {
            let executor = executor.clone();
            let task = task.clone();
            tokio::spawn(async move {
                executor.execute(&task).await
            })
        })
        .collect();

    // Then - 等待所有任务完成
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // 验证所有任务都成功执行（串行）
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}
```

### 6. 实施计划

#### 6.1 分阶段实施策略

**第一阶段：核心架构建立 (2周)**
- ✅ 创建新的模块结构
- ✅ 实现核心Trait (AiExecutor, ToolExecutor)
- ✅ 实现FunctionCallingExecutor基础结构
- ✅ 统一错误处理体系

**第二阶段：工具管理完善 (1.5周)**
- ✅ 实现ToolRegistry
- ✅ 重构ExecutionResult
- ✅ 实现内置Git工具
- ✅ 完善工具权限管理

**第三阶段：会话管理优化 (1周)**
- ✅ 实现SessionManager
- ✅ 完善会话状态管理
- ✅ 添加会话持久化支持

**第四阶段：测试和文档 (1.5周)**
- ✅ 完善测试体系
- ✅ 编写使用文档
- ✅ 性能测试和优化
- ✅ 用户手册和示例

#### 6.2 实施优先级

| 优先级 | 组件 | 预计时间 | 风险等级 | 依赖关系 |
|--------|------|----------|----------|----------|
| 1 | AiExecutor Trait | 3天 | 低 | 无 |
| 2 | ToolExecutor Trait | 3天 | 低 | 1 |
| 3 | FunctionCallingExecutor | 5天 | 高 | 1, 2 |
| 4 | ToolRegistry | 4天 | 中 | 2 |
| 5 | ExecutionResult | 2天 | 低 | 无 |
| 6 | SessionManager | 3天 | 中 | 5 |
| 7 | 测试体系 | 4天 | 低 | 所有 |
| 8 | 文档编写 | 3天 | 低 | 所有 |

### 7. 风险评估和缓解

#### 7.1 技术风险

**高风险：FunctionCallingExecutor重构**
- **风险点**：核心业务逻辑重构可能引入新bug
- **缓解措施**：
  - 保留原有实现作为fallback
  - 分步迁移，每次只重构一个小功能
  - 完善的回归测试覆盖

**中风险：ToolRegistry权限管理**
- **风险点**：权限系统可能过于复杂
- **缓解措施**：
  - 从简单的权限级别开始
  - 渐进式增加复杂性
  - 提供配置选项禁用复杂权限

#### 7.2 兼容性风险

**风险点**：重构可能破坏现有API
- **缓解措施**：
  - 保持现有struct的向后兼容
  - 提供迁移工具和文档
  - 版本化管理，支持平滑升级

#### 7.3 性能风险

**风险点**：新架构可能影响性能
- **缓解措施**：
  - 基准测试对比新旧架构性能
  - 识别性能瓶颈并优化
  - 提供性能监控和调优工具

### 8. 成功指标

#### 8.1 技术指标

- **代码复杂度**：圈复杂度降低30%
- **测试覆盖率**：单元测试 > 85%，集成测试 > 70%
- **构建时间**：减少20%
- **内存使用**：降低15%

#### 8.2 质量指标

- **Bug率**：重构后bug率降低40%
- **可维护性**：代码可读性评分 > 8.0/10
- **扩展性**：新增功能开发时间减少30%

#### 8.3 用户体验指标

- **功能稳定性**：AI功能成功率 > 95%
- **响应时间**：平均响应时间降低25%
- **错误提示**：用户可理解的错误信息 > 90%

### 9. 总结

本次重构将彻底解决当前AI模块的架构问题，建立一个可维护、可扩展、高性能的AI功能体系。通过系统化的模块设计、统一的错误处理、完善的测试体系，我们将为未来的AI功能扩展奠定坚实基础。

**重构的核心价值：**
1. **架构清晰**：职责分离，模块化设计
2. **质量提升**：统一规范，完善测试
3. **性能优化**：并发控制，资源管理
4. **可扩展性**：插件化，标准化接口
5. **用户体验**：错误处理，监控反馈

通过分阶段实施策略，我们可以在控制风险的前提下，逐步实现重构目标，确保项目质量和进度的平衡。

---

**文档版本：v1.0**  
**创建时间：2025-08-25**  
**维护者：Galaxy-flow 开发团队**