# Thread记录功能需求文档

## 文档信息
- **文档名称**: Thread记录功能需求文档
- **创建日期**: 2025-01-14
- **版本**: 1.0
- **状态**: 已确认

## 1. 需求概述

### 1.1 背景
在复杂的软件开发任务中，经常需要与AI进行连续的多次交互来完成一个完整任务。为了保持上下文的连续性和提高AI服务的质量，需要实现Thread记录功能。

### 1.2 目标
- 为连续任务提供上下文保持能力
- 通过手动控制机制，灵活启用Thread记录
- 使用可读性强的格式存储交互历史
- 便于后续的调试、分析和任务跟踪

### 1.3 适用场景
- 代码重构项目
- 复杂系统设计
- 调试和故障排查
- 文档编写和维护
- 其他需要连续交互的复杂任务

## 2. 核心需求

### 2.1 手动控制
- **启用方式**: 用户手动控制Thread记录的开始
- **默认状态**: 默认不启用Thread记录
- **灵活性**: 用户可以根据任务需要决定是否启用

### 2.2 文件存储
- **存储格式**: Markdown文件 (.md)
- **文件命名**: `thread-YYYY-MM-DD.md`
- **生命周期**: 以天为单位，每天一个Thread文件
- **存储位置**: 待确定（需要后续讨论）

### 2.3 记录内容
#### 必须记录的元数据
- **时间戳**: 精确到秒的交互时间
- **模型名称**: 使用的AI模型
- **AI角色**: 当前的AI角色（Developer, Operations, KnowledgeManager）
- **用户请求**: 完整的用户输入内容
- **AI响应**: 总结性内容（非完整响应）

## 3. 详细设计规范

### 3.1 文件结构模板
```markdown
# Thread记录 - YYYY-MM-DD

## 交互记录 1
**时间**: YYYY-MM-DD HH:MM:SS
**模型**: model-name
**角色**: role-name

### 用户请求
```text
用户输入的完整内容...
```

### AI响应（总结）
提取的总结性内容（200-250字）...

## 交互记录 2
**时间**: YYYY-MM-DD HH:MM:SS
**模型**: model-name
**角色**: role-name

### 用户请求
```text
用户输入的完整内容...
```

### AI响应（总结）
提取的总结性内容（200-250字）...
```

### 3.2 总结性内容提取规则

#### 3.2.1 关键字识别
**中文关键字列表**:
- 总结
- 总之
- 综上所述
- 总的来说
- 结论
- 概要
- 摘要
- 最终
- 归纳起来
- 简而言之
- 简单来说
- 简单地讲
- 总体而言
- 整体来看
- 整体而言
- 从整体上
- 总体来说
- 大体而言
- 大体来说
- 基本上
- 基本而言

**英文关键字列表**:
- summary
- conclusion
- in summary
- to summarize
- in conclusion
- to conclude
- overall
- in short
- briefly
- in brief
- essentially
- basically
- ultimately
- finally
- in essence
- to sum up

**识别规则**:
- 不区分大小写
- 支持中英文混合
- 关键字可以出现在段落的任何位置

#### 3.2.2 段落选择逻辑
1. **优先级顺序**:
   - 第一优先级：第一个包含总结关键字的段落
   - 第二优先级：最后一个段落

2. **选择算法**:
   ```rust
   // 伪代码描述
   1. 将AI响应按段落分割
   2. 遍历所有段落，寻找包含总结关键字的段落
   3. 如果找到总结段落：
      - 选择第一个包含总结关键字的段落
      - 应用字数控制规则
   4. 如果没有找到总结段落：
      - 选择最后一个段落
      - 应用字数控制规则
   5. 如果选择的段落太短：
      - 考虑合并相邻段落来满足字数要求
   ```

#### 3.2.3 字数控制规则
- **目标字数范围**: 200-250字
- **截断策略**:
  - 如果段落超过250字，在最近的完整句子处截断
  - 如果段落少于200字，保持原样不额外处理
  - 截断优先在句号(。)、问号(？)、感叹号(！)处进行
  - 保持句子完整性，避免在词中间截断

### 3.3 边界情况处理

#### 3.3.1 多个总结段落
- 优先选择**第一个**包含总结关键字的段落
- 如果第一个总结段落太短，可以考虑合并后续的总结段落

#### 3.3.2 最后段落太短
- 如果最后段落不足200字，按以下顺序处理：
  1. 尝试合并倒数第二段
  2. 如果还是不够，继续向前合并直到满足字数要求
  3. 只合并相邻的段落，不跳跃合并

#### 3.3.3 完全没有合适内容
- 如果所有段落都很短，且找不到总结内容：
  - 选择内容最丰富的段落
  - 或者直接记录最后几个段落的组合

## 4. 具体示例

### 4.1 正常情况示例

#### 示例1：找到总结关键字
**原始AI响应**:
```
这是一个复杂的技术问题，需要从多个角度进行分析。首先，我们需要考虑性能因素，包括算法时间复杂度和空间复杂度。其次，还要考虑代码的可维护性和可扩展性。最后，还需要关注系统的稳定性和安全性。总之，这个问题的解决方案需要综合考虑性能、成本和可维护性等因素，选择最适合当前业务场景的技术方案。
```

**提取的总结性内容**:
```
总之，这个问题的解决方案需要综合考虑性能、成本和可维护性等因素，选择最适合当前业务场景的技术方案。
```

#### 示例2：没有总结关键字
**原始AI响应**:
```
第一步是分析需求和业务场景，明确系统的功能边界。第二步是设计方案架构，选择合适的技术栈和框架。第三步是实施部署，编写代码并进行单元测试。最后，我们需要进行测试验证和性能优化，确保系统稳定运行，并建立完善的监控和告警机制。
```

**提取的总结性内容**:
```
最后，我们需要进行测试验证和性能优化，确保系统稳定运行，并建立完善的监控和告警机制。
```

#### 示例3：字数控制
**原始总结段落**:
```
In summary, this refactoring approach improves code maintainability by separating concerns into distinct modules. The new architecture supports better testability and reduces coupling between components. This results in a more robust and scalable system that can accommodate future changes more easily. Additionally, the modular design allows for better code reuse and easier debugging, making the overall development process more efficient and less error-prone.
```

**提取的总结性内容**:
```
In summary, this refactoring approach improves code maintainability by separating concerns into distinct modules. The new architecture supports better testability and reduces coupling between components. This results in a more robust and scalable system.
```

### 4.2 边界情况示例

#### 示例4：多个总结段落
**原始AI响应**:
```
首先，我们需要对现有代码进行分析和评估。总结来看，代码质量总体良好，但在某些方面存在改进空间。其次，我们应该制定详细的改进计划。总之，建议采用渐进式重构策略，分阶段优化代码结构。
```

**提取的总结性内容**:
```
总结来看，代码质量总体良好，但在某些方面存在改进空间。
```

#### 示例5：最后段落太短
**原始AI响应**:
```
重构的第一步是识别代码中的问题点。第二步是设计新的架构模式。第三步是逐步重构现有代码。第四步是进行测试验证。第五步是性能调优。最后，完成重构。
```

**提取的总结性内容**:
```
第五步是性能调优。最后，完成重构。
```

## 5. 技术实现考虑

### 5.1 集成点
- **AiClient**: 需要在发送请求和接收响应时集成Thread记录功能
- **AiRequest/AiResponse**: 需要支持Thread相关元数据
- **配置系统**: 可能需要添加Thread记录的配置选项

### 5.2 性能考虑
- 文件写入应该是异步的，避免阻塞主线程
- 需要考虑文件并发写入的处理
- 每日文件自动创建和管理机制

### 5.3 错误处理
- 文件写入失败的处理机制
- 文件权限问题的处理
- 磁盘空间不足的处理

## 6. 未来扩展考虑

### 6.1 压缩处理
- 当前版本不实现压缩，为后续版本预留扩展空间
- 可能的压缩方向：
  - 智能摘要生成
  - 重复内容去重
  - 关键信息提取

### 6.2 高级功能
- Thread记录的搜索和查询功能
- Thread记录的可视化展示
- 基于Thread记录的分析和报告生成

## 7. 验收标准

### 7.1 功能验收
- [ ] 手动控制Thread记录启用/禁用
- [ ] 每日自动创建Thread文件
- [ ] 正确记录所有必需的元数据
- [ ] 准确提取总结性内容
- [ ] 正确应用字数控制规则
- [ ] 处理各种边界情况

### 7.2 质量验收
- [ ] 生成的Thread文件格式正确且可读
- [ ] 总结性内容提取准确度高
- [ ] 文件写入稳定可靠
- [ ] 不影响现有功能的性能

## 8. 风险和依赖

### 8.1 技术风险
- 文件I/O性能可能影响整体响应时间
- 并发写入可能导致文件内容混乱
- 磁盘空间可能成为瓶颈

### 8.2 依赖关系
- 依赖于文件系统的权限管理
- 依赖于时间戳的准确性
- 依赖于现有的AI请求/响应流程

## 9. 技术实现方案

### 9.1 AI通知功能扩展

在原有的Thread记录功能基础上，增加了AI通知功能，允许用户选择是否告知AI当前对话正在被记录。

#### 9.1.1 设计原理
- **非侵入性设计**: AI通知作为可选功能，不影响原有的Thread记录流程
- **安全性考虑**: 通过系统提示通知AI，避免了循环调用的问题
- **灵活性**: 用户可以自定义通知消息，满足不同场景需求
- **向后兼容**: 默认关闭AI通知，不影响现有使用方式

#### 9.1.2 实现机制
AI通知通过在系统提示中添加特定的通知消息来实现：
```rust
// 原始系统提示
"你是AI助手"

// 启用AI通知后的系统提示
"你是AI助手

【Thread记录已启用】本次对话正在被记录，请确保回答内容适合记录和分析。"
```

这样AI能够了解当前对话的记录状态，并相应调整其回答内容和方式。

### 9.1 总体架构设计

采用**装饰器模式 + 简单文件追加 + 配置文件控制**的组合方案，将Thread配置直接集成到现有的`ai.yaml`配置文件中。

### 9.2 配置集成设计

#### 9.2.1 配置结构设计

Thread配置作为`AiConfig`结构体的一个字段直接集成：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfig {
    pub providers: HashMap<AiProviderType, ProviderConfig>,
    
    /// Thread记录配置
    #[serde(default = "default_thread_config")]
    pub thread: ThreadConfig,
    
    pub router: RouterConfig,
    pub roles: RolesConfig,
}

/// Thread记录配置结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadConfig {
    /// 是否启用Thread记录
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    
    /// Thread文件存储路径
    #[serde(default = "default_storage_path")]
    pub storage_path: PathBuf,
    
    /// 文件名模板
    #[serde(default = "default_filename_template")]
    pub filename_template: String,
    
    /// 最小总结字数
    #[serde(default = "default_min_summary_length")]
    pub min_summary_length: usize,
    
    /// 最大总结字数
    #[serde(default = "default_max_summary_length")]
    pub max_summary_length: usize,
    
    /// 总结关键字列表
    #[serde(default = "default_thread_summary_keywords")]
    pub summary_keywords: Vec<String>,

    /// 是否告知AI正在被记录
    #[serde(default = "default_thread_inform_ai")]
    pub inform_ai: bool,

    /// 告知AI的通知消息
    #[serde(default = "default_thread_inform_message")]
    pub inform_message: String,
  }
```

#### 9.2.2 完整的ai.yaml配置示例

```yaml
# ai.yaml - 完整配置示例

# AI提供者配置
providers:
  openai:
    enabled: true
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    models:
      - "gpt-4o"
      - "gpt-4o-mini"
      - "gpt-3.5-turbo"
  
  deepseek:
    enabled: true
    api_key: "${DEEPSEEK_API_KEY}"
    base_url: "https://api.deepseek.com"
    models:
      - "deepseek-chat"
      - "deepseek-coder"
  
  groq:
    enabled: true
    api_key: "${GROQ_API_KEY}"
    base_url: "https://api.groq.com/openai/v1"
    models:
      - "llama3-70b-8192"
      - "llama3-8b-8192"
      - "mixtral-8x7b-32768"

# Thread记录配置
thread:
  enabled: true                    # 启用Thread记录
  storage_path: "./threads"        # Thread文件存储路径
  filename_template: "thread-YYYY-MM-DD.md"  # 文件名模板
  min_summary_length: 200          # 最小总结字数
  max_summary_length: 250          # 最大总结字数
  
  # 总结关键字配置（可覆盖默认值）
  summary_keywords:
    - "总结"
    - "总之"
    - "结论"
    - "summary"
    - "conclusion"
    - "in summary"
  
  # AI通知配置
  inform_ai: false                 # 是否告知AI正在被记录
  inform_message: "【Thread记录已启用】本次对话正在被记录，请确保回答内容适合记录和分析。"  # 通知消息
  
  # AI通知配置
  inform_ai: false                 # 是否告知AI正在被记录
  inform_message: "【Thread记录已启用】本次对话正在被记录，请确保回答内容适合记录和分析。"  # 通知消息

# 路由配置
router:
  default_provider: "openai"
  model_mappings:
    "gpt-4o": "openai"
    "gpt-4o-mini": "openai"
    "deepseek-chat": "deepseek"
    "deepseek-coder": "deepseek"

# 角色配置
roles:
  enabled: true
  config_path: "./config/roles"
```

### 9.3 装饰器模式实现

### 9.3.1 核心架构
```rust
/// AI客户端trait，用于装饰器模式
pub trait AiClientTrait: Send + Sync {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse>;
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse>;
}

/// Thread记录装饰器
pub struct ThreadRecordingClient<T: AiClientTrait> {
    inner: Arc<T>,                        // 被装饰的AiClient
    config: Arc<ThreadConfig>,            // Thread配置
    file_manager: Arc<ThreadFileManager>, // 文件管理器
}
```

### 9.3.2 AI通知功能实现
```rust
impl<T: AiClientTrait> ThreadRecordingClient<T> {
    /// 构建带有Thread通知的请求
    fn build_request_with_thread_info(&self, mut request: AiRequest) -> AiRequest {
        if self.config.inform_ai {
            // 在系统提示中添加Thread记录通知
            request.system_prompt = format!(
                "{}\n\n{}",
                request.system_prompt, self.config.inform_message
            );
        }
        request
    }
}
```

当`inform_ai`设置为`true`时，系统会自动在AI请求的系统提示中添加通知消息，让AI知道当前对话正在被记录。

impl<T: AiClientTrait> ThreadRecordingClient<T> {
    pub fn new(inner: T, config: ThreadConfig) -> Self {
        Self {
            inner: Arc::new(inner),
            config: Arc::new(config),
            file_manager: Arc::new(ThreadFileManager::new(config.clone())),
        }
    }
    
    /// 检查是否启用Thread记录
    fn is_thread_enabled(&self) -> bool {
        self.config.enabled
    }
}

impl<T: AiClientTrait> AiClientTrait for ThreadRecordingClient<T> {
    async fn send_request(&self, request: AiRequest) -> AiResult<AiResponse> {
        let start_time = Utc::now();
        
        // 调用内部客户端
        let response = self.inner.send_request(request.clone()).await;
        
        // 如果启用Thread记录且响应成功，则记录交互
        if self.is_thread_enabled() {
            if let Ok(ref resp) = response {
                if let Err(e) = self.file_manager.record_interaction(
                    start_time,
                    &request,
                    resp,
                ).await {
                    // 记录失败不应该影响主要功能，只记录警告
                    eprintln!("Warning: Failed to record thread interaction: {}", e);
                }
            }
        }
        
        response
    }
    
    async fn smart_role_request(&self, role: AiRole, user_input: &str) -> AiResult<AiResponse> {
        let start_time = Utc::now();
        
        // 调用内部客户端
        let response = self.inner.smart_role_request(role, user_input).await;
        
        // 如果启用Thread记录且响应成功，则记录交互
        if self.is_thread_enabled() {
            if let Ok(ref resp) = response {
                // 为smart_role_request构建等效的AiRequest用于记录
                let model = role.recommended_model();
                let system_prompt = format!("角色: {}", role.description());
                
                let request = AiRequest::builder()
                    .model(model)
                    .system_prompt(system_prompt)
                    .user_prompt(user_input.to_string())
                    .role(role)
                    .build();
                
                if let Err(e) = self.file_manager.record_interaction(
                    start_time,
                    &request,
                    resp,
                ).await {
                    eprintln!("Warning: Failed to record thread interaction: {}", e);
                }
            }
        }
        
        response
    }
}
```

### 9.4 文件管理器实现

#### 9.4.1 ThreadFileManager核心功能

```rust
pub struct ThreadFileManager {
    config: Arc<ThreadConfig>,
    interaction_counter: AtomicUsize,  // 交互计数器
    base_path: PathBuf,               // 基础路径
}

impl ThreadFileManager {
    pub fn new(config: ThreadConfig) -> Self {
        let base_path = Self::resolve_storage_path(&config.storage_path);
        
        Self {
            config: Arc::new(config),
            interaction_counter: AtomicUsize::new(1),
            base_path,
        }
    }
    
    pub async fn record_interaction(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
        request: &AiRequest,
        response: &AiResponse,
    ) -> AiResult<()> {
        // 1. 生成今日文件路径
        let file_path = self.generate_daily_file_path(&timestamp);
        
        // 2. 确保目录存在
        self.ensure_directory_exists(&file_path)?;
        
        // 3. 提取总结性内容
        let summary_content = self.extract_summary_content(&response.content);
        
        // 4. 格式化记录内容
        let interaction_number = self.interaction_counter.fetch_add(1, Ordering::SeqCst);
        let record_content = self.format_interaction_record(
            timestamp,
            interaction_number,
            request,
            &summary_content,
        );
        
        // 5. 追加写入文件
        self.append_to_file(&file_path, &record_content).await
    }
    
    fn generate_daily_file_path(&self, timestamp: &chrono::DateTime<chrono::Utc>) -> PathBuf {
        let date_str = timestamp.format("%Y-%m-%d").to_string();
        let filename = self.config.filename_template
            .replace("YYYY-MM-DD", &date_str);
        
        // 确保文件名以.md结尾
        let filename = if filename.ends_with(".md") {
            filename
        } else {
            format!("{}.md", filename)
        };
        
        self.base_path.join(filename)
    }
    
    fn format_interaction_record(
        &self,
        timestamp: chrono::DateTime<chrono::Utc>,
        interaction_number: usize,
        request: &AiRequest,
        summary_content: &str,
    ) -> String {
        let role_str = request.role.map_or("None".to_string(), |r| r.to_string());
        
        format!(
            "## 交互记录 {}\n**时间**: {}\n**模型**: {}\n**角色**: {}\n\n### 用户请求\n```text\n{}```\n\n### AI响应（总结）\n{}\n\n",
            interaction_number,
            timestamp.format("%Y-%m-%d %H:%M:%S"),
            request.model,
            role_str,
            request.user_prompt,
            summary_content
        )
    }
    
    async fn append_to_file(&self, path: &Path, content: &str) -> AiResult<()> {
        let mut file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await
            .map_err(|e| AiError::from(AiErrReason::IoError(format!(
                "Failed to open file {}: {}", 
                path.display(), e
            ))))?;
        
        file.write_all(content.as_bytes()).await
            .map_err(|e| AiError::from(AiErrReason::IoError(format!(
                "Failed to write to file {}: {}", 
                path.display(), e
            ))))?;
        
        file.flush().await
            .map_err(|e| AiError::from(AiErrReason::IoError(format!(
                "Failed to flush file {}: {}", 
                path.display(), e
            ))))?;
        
        Ok(())
    }
}
```

### 9.5 总结性内容提取器实现

```rust
pub struct SummaryExtractor {
    keywords: Vec<String>,
    min_length: usize,
    max_length: usize,
}

impl SummaryExtractor {
    pub fn new(keywords: &[String]) -> Self {
        Self {
            keywords: keywords.to_vec(),
            min_length: 200,
            max_length: 250,
        }
    }
    
    pub fn extract_with_length_limits(&self, content: &str, min_len: usize, max_len: usize) -> String {
        let paragraphs: Vec<&str> = content.split('\n').filter(|p| !p.trim().is_empty()).collect();
        
        // 1. 寻找包含总结关键字的段落
        if let Some(summary_paragraph) = self.find_summary_paragraph(&paragraphs) {
            return self.truncate_to_length(summary_paragraph, max_len);
        }
        
        // 2. 如果没有找到，使用最后一段
        if let Some(last_paragraph) = paragraphs.last() {
            return self.truncate_to_length(last_paragraph, max_len);
        }
        
        // 3. 如果没有段落，返回空字符串
        String::new()
    }
    
    fn find_summary_paragraph(&self, paragraphs: &[&str]) -> Option<&str> {
        for paragraph in paragraphs {
            if self.contains_summary_keyword(paragraph) {
                return Some(paragraph);
            }
        }
        None
    }
    
    fn contains_summary_keyword(&self, text: &str) -> bool {
        let lower_text = text.to_lowercase();
        self.keywords.iter().any(|keyword| {
            lower_text.contains(&keyword.to_lowercase())
        })
    }
    
    fn truncate_to_length(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            return text.to_string();
        }
        
        // 在句子边界处截断
        let truncated = &text[..max_len];
        if let Some(last_sentence_end) = self.find_last_sentence_end(truncated) {
            truncated[..last_sentence_end].to_string()
        } else {
            truncated.to_string()
        }
    }
    
    fn find_last_sentence_end(&self, text: &str) -> Option<usize> {
        // 寻找最后一个句号、问号或感叹号
        let sentence_endings = ['。', '？', '！', '.', '?', '!'];
        for (i, c) in text.char_indices().rev() {
            if sentence_endings.contains(&c) {
                return Some(i + c.len_utf8());
            }
        }
        None
    }
}
```

### 9.6 集成到AiClient创建流程

```rust
impl AiClient {
    /// 创建支持Thread记录的AiClient
    pub fn new_with_thread_support(config: AiConfig) -> AiResult<Box<dyn AiClientTrait>> {
        let base_client = Self::new(config.clone())?;
        
        // 根据配置决定是否使用装饰器
        if config.thread.enabled {
            let decorated_client = ThreadRecordingClient::new(base_client, config.thread);
            Ok(Box::new(decorated_client))
        } else {
            Ok(Box::new(base_client))
        }
    }
    
    /// 保持原有接口不变，内部透明使用装饰器
    pub fn new(config: AiConfig) -> AiResult<Self> {
        // 现有实现...
    }
}
```

### 9.7 配置加载和验证

```rust
impl AiConfig {
    /// 从默认路径加载配置
    pub fn load() -> AiResult<Self> {
        let config_path = Self::find_config_path()
            .ok_or_else(|| AiErrReason::ConfigError("Cannot find ai.yaml".to_string()))?;
        
        let content = std::fs::read_to_string(&config_path)
            .map_err(|e| AiErrReason::ConfigError(format!("Failed to read config file: {}", e)))?;
        
        let mut config: AiConfig = serde_yaml::from_str(&content)
            .map_err(|e| AiErrReason::ConfigError(format!("Failed to parse YAML: {}", e)))?;
        
        // 验证和后处理配置
        config.validate_and_postprocess()?;
        
        Ok(config)
    }
    
    /// 验证和后处理配置
    fn validate_and_postprocess(&mut self) -> AiResult<()> {
        // 验证Thread配置
        self.validate_thread_config()?;
        
        // 其他配置验证...
        
        Ok(())
    }
    
    /// 验证Thread配置
    fn validate_thread_config(&mut self) -> AiResult<()> {
        // 验证存储路径
        if self.thread.storage_path.as_os_str().is_empty() {
            self.thread.storage_path = PathBuf::from("./threads");
        }
        
        // 验证字数范围的合理性
        if self.thread.min_summary_length == 0 {
            self.thread.min_summary_length = 200;
        }
        if self.thread.max_summary_length == 0 {
            self.thread.max_summary_length = 250;
        }
        
        // 确保最小值不大于最大值
        if self.thread.min_summary_length > self.thread.max_summary_length {
            return Err(AiErrReason::ConfigError(
                "Thread min_summary_length cannot be greater than max_summary_length".to_string()
            ));
        }
        
        // 验证关键字列表不为空
        if self.thread.summary_keywords.is_empty() {
            self.thread.summary_keywords = default_summary_keywords();
        }
        
        // 去重关键字
        self.thread.summary_keywords.sort();
        self.thread.summary_keywords.dedup();
        
        Ok(())
    }
}
```

---

**文档状态**: 技术方案设计完成，准备进入实现阶段

**下一步**: 
1. 实现Thread配置结构体和默认值 ✅
2. 实现ThreadRecordingClient装饰器 ✅
3. 实现ThreadFileManager和SummaryExtractor ✅
4. 集成到现有的AiClient创建流程 ✅
5. 编写单元测试和集成测试 ✅
6. 实现AI通知功能 ✅
7. 验证完整功能 ✅

**功能状态**: 完全实现并通过所有测试