#  Galaxy Flow

Galaxy Flow is a natively AI-integrated DSL for process orchestration, designed for intelligent workflow execution across OPS, DevOps, and SecOps domains.

[![GitHub Actions](https://github.com/galaxy-sec/galaxy-flow/workflows/check/badge.svg)](https://github.com/galaxy-sec/galaxy-flow/actions?query=workflow%3Acheck)
[![Coverage Status](https://coveralls.io/repos/github/galaxy-sec/galaxy-flow/badge.svg)](https://coveralls.io/github/galaxy-sec/galaxy-flow)

Galaxy Flow 是一个 原生AI集成的流程编排领域专用语言，适用于 OPS、DevOps、SecOps 等多领域的流程智能化执行。

## 核心能力 / Core Capabilities

### AI 集成能力 / AI Integration Capabilities

**中文版本：**
通过 `gx.ai_chat` 组件，GFlow 提供了强大的 AI 对话功能：
- 支持直接消息提示和文件提示两种输入方式
- 集成多种 AI 提供商配置，支持灵活的 AI 服务选择
- 提供 AI 角色管理功能，可以指定不同的 AI 角色进行专门对话
- 支持从环境变量动态加载 AI 配置，便于部署和管理
- 异步执行 AI 请求，并将响应结果输出到控制台

**English Version:**
Through the `gx.ai_chat` component, GFlow provides powerful AI conversation capabilities:
- Supports both direct message prompts and file prompt input methods
- Integrates multiple AI provider configurations for flexible AI service selection
- Provides AI role management functionality, allowing specification of different AI roles for specialized conversations
- Supports dynamic loading of AI configurations from environment variables for easy deployment and management
- Asynchronous execution of AI requests with response results output to console

### 智能诊断能力 / Intelligent Diagnosis Capabilities

**中文版本：**
通过 `ai_diagnose` 函数提供 AI 驱动的诊断分析：
- 自动读取运行输出日志和 `.gxl` 工作流文件
- 将运行信息和 GXL 文件内容发送给 AI 进行智能分析
- 使用特定 AI 角色（如 "galactiward"）进行专业的诊断分析
- 提供详细的分析结果和建议，包括内容、模型信息和时间戳
- 帮助用户快速定位和解决工作流执行中的问题

**English Version:**
Through the `ai_diagnose` function, GFlow provides AI-driven diagnostic analysis:
- Automatically reads execution output logs and `.gxl` workflow files
- Sends execution information and GXL file content to AI for intelligent analysis
- Uses specific AI roles (such as "galactiward") for professional diagnostic analysis
- Provides detailed analysis results and suggestions, including content, model information, and timestamps
- Helps users quickly identify and resolve issues in workflow execution

### 核心执行能力 / Core Execution Capabilities

**中文版本：**
GFlow 提供了丰富的执行能力组件，支持复杂的 DevSecOps 场景：
- **Shell 执行能力** (`gx.shell`): 支持命令行执行，支持参数文件（JSON、YAML、TOML、INI格式），输出变量捕获
- **文件操作能力**: 提供文件读取、上传、下载功能，支持多种文件操作场景
- **模板渲染能力**: 支持模板渲染和变量替换，便于动态内容生成
- **断言验证能力**: 提供条件验证和结果检查，确保工作流执行的可靠性
- **工作流运行能力**: 支持 GXL 工作流的递归执行，实现复杂流程编排
- **归档处理能力**: 支持文件压缩和解压操作，便于文件管理和传输
- **委托执行能力**: 支持任务委托和分布式执行，提高执行效率

**English Version:**
GFlow provides rich execution capability components, supporting complex DevSecOps scenarios:
- **Shell Execution Capability** (`gx.shell`): Supports command-line execution with parameter files (JSON, YAML, TOML, INI formats) and output variable capture
- **File Operations**: Provides file reading, upload, and download functionality for various file operation scenarios
- **Template Rendering**: Supports template rendering and variable replacement for dynamic content generation
- **Assertion Validation**: Provides condition validation and result checking to ensure workflow execution reliability
- **Workflow Execution**: Supports recursive execution of GXL workflows for complex process orchestration
- **Archive Processing**: Supports file compression and decompression operations for easy file management and transfer
- **Delegated Execution**: Supports task delegation and distributed execution to improve execution efficiency

## docs
* [git-docs](https://galaxy-sec.github.io/gxl-docs/)
* [deepwiki](https://deepwiki.com/galaxy-sec/galaxy-flow)

##  下载
项目的正式发布版本可在GitHub发布页面获取：

https://github.com/galaxy-sec/galaxy-flow/releases

## 命令行工具

### 核心命令
```
gflow
gprj
```

#### gflow
对项目定义的工作流（ work.gxl） 运行

#### gprj
对项目定义的管理流（ adm.gxl） 运行

![](./images/command-line.jpg)
