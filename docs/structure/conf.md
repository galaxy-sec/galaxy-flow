# conf 模块结构文档

## 模块概述

conf 模块是 galaxy-flow 的配置管理核心，负责处理 GXL 项目的配置文件、环境变量、命令行参数等各种配置源。它提供了统一的配置接口，支持多环境配置、配置继承、动态配置更新等功能。

## 模块结构图

```mermaid
graph TD
    A[conf] --> B[base_env]
    A --> C[cli]
    A --> D[envs]
    A --> E[file]
    A --> F[loader]
    A --> G[merge]
    A --> H[model]
    A --> I[parser]
    A --> J[resolver]
    A --> K[source]
    A --> L[types]
    A --> M[util]
    A --> N[validate]

    C --> C1[args]
    C --> C2[command]
    C --> C3[help]
    C --> C4[options]

    E --> E1[json]
    E --> E2[toml]
    E --> E3[yaml]

    F --> F1[config_loader]
    F --> F2[env_loader]
    F --> F3[file_loader]

    K --> K1[config_source]
    K --> K2[env_source]
    K --> K3[file_source]

    style A fill:#f9f,stroke:#333
    style C fill:#bbf,stroke:#333
    style E fill:#bfb,stroke:#333
    style F fill:#ffb,stroke:#333
    style K fill:#fbf,stroke:#333
```

## 配置架构

### 整体配置流程

```mermaid
graph LR
    A[配置源] --> B[配置加载器]
    B --> C[配置解析器]
    C --> D[配置合并器]
    D --> E[配置验证器]
    E --> F[配置解析器]
    F --> G[配置模型]

    A --> A1[文件配置]
    A --> A2[环境变量]
    A --> A3[命令行参数]
    A --> A4[默认值]

    style A fill:#f9f,stroke:#333
    style G fill:#bfb,stroke:#333
```

## 核心配置组件

### 1. 基础环境 (base_env)
定义基础环境配置模板。

**主要功能：**
- 基础环境定义
- 通用配置模板
- 环境继承机制
- 默认配置值

### 2. 命令行接口 (cli)
处理命令行参数和选项。

**子模块：**
- **args**: 参数解析
- **command**: 命令定义
- **help**: 帮助信息
- **options**: 选项处理

### 3. 环境配置 (envs)
管理不同环境的配置。

**主要功能：**
- 环境定义
- 环境切换
- 环境继承
- 环境变量映射

### 4. 文件配置 (file)
处理各种格式的配置文件。

**支持的格式：**
- **JSON**: 结构化配置
- **TOML**: 人类可读配置
- **YAML**: 层次化配置

### 5. 配置加载器 (loader)
统一配置加载接口。

**主要加载器：**
- **config_loader**: 通用配置加载
- **env_loader**: 环境变量加载
- **file_loader**: 文件配置加载

## 配置源管理

### 配置源结构

```mermaid
graph TD
    A[配置源] --> B[文件源]
    A --> C[环境源]
    A --> D[命令行源]
    A --> E[内存源]

    B --> B1[JSON文件]
    B --> B2[TOML文件]
    B --> B3[YAML文件]

    C --> C1[系统环境变量]
    C --> C2[进程环境变量]

    D --> D1[位置参数]
    D --> D2[命名参数]
    D --> D3[标志参数]

    style A fill:#fbf,stroke:#333
    style B fill:#bbf,stroke:#333
    style C fill:#bfb,stroke:#333
    style D fill:#ffb,stroke:#333
```

### 1. 配置源 (config_source)
定义配置数据的来源。

**主要功能：**
- 配置源接口
- 优先级管理
- 动态更新
- 缓存机制

### 2. 环境源 (env_source)
从环境变量获取配置。

**主要功能：**
- 环境变量读取
- 前缀过滤
- 类型转换
- 默认值处理

### 3. 文件源 (file_source)
从配置文件获取配置。

**主要功能：**
- 文件监控
- 格式检测
- 热重载
- 错误处理

## 配置合并策略

### 合并规则

```mermaid
graph LR
    A[配置合并] --> B[优先级规则]
    A --> C[覆盖规则]
    A --> D[合并规则]
    A --> E[验证规则]

    B --> B1[命令行 > 环境 > 文件 > 默认]
    C --> C1[高优先级覆盖低优先级]
    D --> D1[深度合并对象]
    E --> E1[类型验证]

    style A fill:#ffb,stroke:#333
    style B fill:#bbf,stroke:#333
    style C fill:#bfb,stroke:#333
    style E fill:#fbb,stroke:#333
```

### 1. 合并器 (merge)
处理配置的合并逻辑。

**主要功能：**
- 深度合并
- 数组合并
- 对象合并
- 冲突解决

### 2. 验证器 (validate)
验证配置的有效性。

**主要功能：**
- 类型验证
- 范围验证
- 依赖验证
- 格式验证

## 配置模型

### 配置数据结构

```mermaid
graph TD
    A[配置模型] --> B[基础配置]
    A --> C[环境配置]
    A --> D[项目配置]
    A --> E[用户配置]

    B --> B1[日志级别]
    B --> B2[调试模式]
    B --> B3[超时设置]

    C --> C1[开发环境]
    C --> C2[测试环境]
    C --> C3[生产环境]

    D --> D1[项目名称]
    D --> D2[版本信息]
    D --> D3[依赖配置]

    style A fill:#f9f,stroke:#333
    style B fill:#bbf,stroke:#333
    style C fill:#bfb,stroke:#333
    style D fill:#ffb,stroke:#333
```

### 1. 配置模型 (model)
定义配置的数据结构。

**主要结构：**
- 基础配置
- 环境配置
- 项目配置
- 用户配置
- 系统配置

### 2. 配置解析器 (parser)
解析配置文件内容。

**主要功能：**
- 语法解析
- 语义分析
- 错误报告
- 位置信息

## 配置解析器

### 解析器架构

```mermaid
graph TD
    A[配置解析器] --> B[JSON解析器]
    A --> C[TOML解析器]
    A --> D[YAML解析器]
    A --> E[环境变量解析器]
    A --> F[命令行解析器]

    B --> B1[语法验证]
    C --> C1[语义验证]
    D --> D1[结构验证]
    E --> E1[格式验证]
    F --> F1[参数验证]

    style A fill:#fbf,stroke:#333
    style B fill:#bbf,stroke:#333
    style C fill:#bfb,stroke:#333
    style D fill:#ffb,stroke:#333
```

## 配置工具

### 1. 配置工具 (util)
提供配置处理的工具函数。

**主要功能：**
- 路径处理
- 默认值设置
- 类型转换
- 格式化输出

### 2. 配置解析器 (resolver)
解析配置引用和变量。

**主要功能：**
- 变量替换
- 配置引用
- 环境变量解析
- 路径解析

## 配置类型

### 配置类型系统

```rust
pub enum ConfigValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Array(Vec<ConfigValue>),
    Object(HashMap<String, ConfigValue>),
    Null,
}

pub struct Config {
    pub base: BaseConfig,
    pub environments: HashMap<String, EnvironmentConfig>,
    pub project: ProjectConfig,
    pub user: UserConfig,
}
```

## 使用示例

### 1. 基础配置加载

```rust
use crate::conf::{
    loader::ConfigLoader,
    model::Config,
    source::ConfigSource
};

// 创建配置加载器
let loader = ConfigLoader::new();

// 添加配置源
loader.add_source(ConfigSource::File("config.toml"));
loader.add_source(ConfigSource::Env);
loader.add_source(ConfigSource::Args);

// 加载配置
let config = loader.load()?;
```

### 2. 环境配置

```rust
use crate::conf::{
    envs::Environment,
    loader::ConfigLoader
};

// 设置环境
let env = Environment::Development;

// 加载环境配置
let config = ConfigLoader::new()
    .with_environment(env)
    .load()?;
```

### 3. 命令行配置

```rust
use crate::conf::cli::{
    Args,
    Command,
    Options
};

// 解析命令行参数
let args = Args::parse();
let command = Command::from_args(&args);
let options = Options::from_args(&args);
```

## 配置验证

### 验证规则

```rust
pub struct ValidationRule {
    pub field: String,
    pub rule_type: ValidationType,
    pub required: bool,
    pub default: Option<ConfigValue>,
}

pub enum ValidationType {
    Type(ConfigType),
    Range(f64, f64),
    Length(usize, usize),
    Pattern(String),
    Enum(Vec<String>),
}
```

## 依赖关系

```mermaid
graph LR
    conf --> model
    conf --> util
    conf --> err
    conf --> types
    
    cli --> args
    file --> json
    file --> toml
    file --> yaml
    
    loader --> source
    resolver --> util
    
    style conf fill:#f9f
    style model fill:#fbf
    style util fill:#bbf
```

## 扩展指南

要添加新的配置功能：

1. 在相应目录创建新的配置模块
2. 实现 ConfigSource trait
3. 添加配置验证规则
4. 更新配置模型
5. 添加单元测试
6. 更新文档和示例

## 测试策略

- **单元测试**: 测试单个配置组件
- **集成测试**: 测试配置加载流程
- **环境测试**: 测试不同环境下的配置
- **格式测试**: 测试不同配置格式的解析
- **验证测试**: 测试配置验证规则
- **性能测试**: 测试配置加载性能