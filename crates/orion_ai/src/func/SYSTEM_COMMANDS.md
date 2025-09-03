# Orion AI 系统命令使用文档

本文档详细介绍了 Orion AI 系统中可用的系统命令工具，包括文件系统操作、系统信息查询和网络工具。

## 概述

Orion AI 系统命令模块提供了安全、受控的 Linux/macOS 常用命令访问能力，通过分类执行器模式实现，包含三个主要执行器：

- **FileSystemExecutor**: 文件系统操作（`fs-` 前缀）
- **SystemInfoExecutor**: 系统信息获取（`sys-` 前缀）  
- **NetworkExecutor**: 网络工具（`net-` 前缀）

所有命令都经过安全验证，支持超时控制，防止恶意操作和资源滥用。

## 安全特性

### 路径安全
- **目录遍历防护**: 禁止 `../` 和 `~` 路径
- **命令注入防护**: 过滤特殊字符 `; | & $ > <`
- **路径规范化**: 自动处理和验证路径格式

### 执行控制
- **超时限制**: 每个命令都有执行超时控制
- **输出限制**: 防止过大输出消耗资源
- **参数验证**: 严格的参数格式和范围检查

### 网络安全
- **主机名验证**: 防止网络命令注入
- **连接限制**: 限制 ping 包数量和超时时间
- **目标过滤**: 防止访问恶意目标

## 文件系统命令 (fs-)

### fs-ls - 列出目录内容

列出指定目录的文件和子目录。

**函数定义**:
```json
{
  "name": "fs-ls",
  "description": "列出目录内容",
  "parameters": [
    {
      "name": "path",
      "description": "要列出的目录路径，默认为当前目录",
      "type": "string",
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "fs-ls",
  "arguments": "{\"path\": \"/tmp\"}"
}
```

**返回结果**:
```json
{
  "name": "fs-ls",
  "result": {
    "path": "/tmp",
    "output": "total 8\ndrwxrwxrwt 2 root root 160 Jan 15 10:30 .\ndrwxr-xr-x 22 root root 680 Jan 15 09:00 ..\n-rw-r--r-- 1 user user 0 Jan 15 10:30 test.txt",
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 不支持目录遍历 (`../`)
- 默认显示当前目录内容
- 使用 `ls -la` 命令格式

---

### fs-pwd - 显示当前工作目录

显示当前工作目录的完整路径。

**函数定义**:
```json
{
  "name": "fs-pwd", 
  "description": "显示当前工作目录",
  "parameters": []
}
```

**使用示例**:
```json
{
  "name": "fs-pwd",
  "arguments": "{}"
}
```

**返回结果**:
```json
{
  "name": "fs-pwd",
  "result": {
    "current_directory": "/home/user/projects",
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 无需参数
- 总是返回绝对路径

---

### fs-cat - 显示文件内容

读取并显示指定文件的内容。

**函数定义**:
```json
{
  "name": "fs-cat",
  "description": "显示文件内容", 
  "parameters": [
    {
      "name": "path",
      "description": "要读取的文件路径",
      "type": "string",
      "required": true
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "fs-cat",
  "arguments": "{\"path\": \"/etc/hostname\"}"
}
```

**返回结果**:
```json
{
  "name": "fs-cat", 
  "result": {
    "path": "/etc/hostname",
    "content": "my-computer\n",
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 必须提供文件路径参数
- 只能读取文本文件
- 不支持目录遍历
- 有读取权限限制

---

### fs-find - 查找文件

在指定路径下查找匹配模式的文件。

**函数定义**:
```json
{
  "name": "fs-find",
  "description": "查找文件",
  "parameters": [
    {
      "name": "path",
      "description": "搜索的起始路径，默认为当前目录", 
      "type": "string",
      "required": false
    },
    {
      "name": "pattern",
      "description": "文件名模式，支持通配符，默认为 *",
      "type": "string", 
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "fs-find",
  "arguments": "{\"path\": \"/home/user\", \"pattern\": \"*.txt\"}"
}
```

**返回结果**:
```json
{
  "name": "fs-find",
  "result": {
    "path": "/home/user", 
    "pattern": "*.txt",
    "files": [
      "/home/user/doc.txt",
      "/home/user/notes.txt"
    ],
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 支持通配符模式（`*`, `?`）
- 递归搜索子目录
- 执行超时30秒
- 不支持目录遍历

## 系统信息命令 (sys-)

### sys-uname - 显示系统信息

显示操作系统和系统硬件信息。

**函数定义**:
```json
{
  "name": "sys-uname",
  "description": "显示系统信息",
  "parameters": [
    {
      "name": "detailed",
      "description": "是否显示详细信息，默认为false", 
      "type": "boolean",
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "sys-uname",
  "arguments": "{\"detailed\": true}"
}
```

**返回结果**:
```json
{
  "name": "sys-uname",
  "result": {
    "output": "Darwin my-computer 23.2.0 Darwin Kernel Version 23.2.0: Wed Nov 15 21:53:34 PST 2023; root:xnu-10002.61.3~2/RELEASE_ARM64_T6000 arm64",
    "detailed": true,
    "success": true
  },
  "error": null
}
```

**注意事项**:
- `detailed=false` 时只显示系统名称
- `detailed=true` 时显示所有系统信息
- 执行超时10秒

---

### sys-ps - 显示进程信息

显示当前运行的进程信息。

**函数定义**:
```json
{
  "name": "sys-ps", 
  "description": "显示进程信息",
  "parameters": [
    {
      "name": "process_name",
      "description": "按进程名过滤，可选",
      "type": "string",
      "required": false
    },
    {
      "name": "user", 
      "description": "按用户名过滤，可选",
      "type": "string", 
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "sys-ps",
  "arguments": "{\"process_name\": \"nginx\"}"
}
```

**返回结果**:
```json
{
  "name": "sys-ps",
  "result": {
    "processes": [
      "USER         PID %CPU %MEM    VSZ   RSS TTY      STAT START   TIME COMMAND",
      "root        1234  0.0  0.1  12345  6789 ?        Ss   Jan15   0:01 nginx: master process",
      "www-data   1235  0.0  0.0  12346  6790 ?        S    Jan15   0:00 nginx: worker process"
    ],
    "process_filter": "nginx",
    "user_filter": null,
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 使用 `ps aux` 命令格式
- 支持进程名和用户名过滤
- 执行超时15秒
- 返回所有匹配的进程行

---

### sys-df - 显示磁盘使用情况

显示文件系统的磁盘空间使用情况。

**函数定义**:
```json
{
  "name": "sys-df",
  "description": "显示磁盘使用情况", 
  "parameters": [
    {
      "name": "path",
      "description": "要检查的路径，默认为当前目录",
      "type": "string",
      "required": false
    },
    {
      "name": "human_readable",
      "description": "是否使用人类可读格式，默认为true",
      "type": "boolean", 
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "sys-df",
  "arguments": "{\"path\": \"/\", \"human_readable\": true}"
}
```

**返回结果**:
```json
{
  "name": "sys-df",
  "result": {
    "disk_usage": [
      "Filesystem      Size  Used Avail Use% Mounted on",
      "/dev/disk1s1   500G  200G  300G  40% /",
      "devfs          328K  328K    0B 100% /dev"
    ],
    "path": "/",
    "human_readable": true, 
    "success": true
  },
  "error": null
}
```

**注意事项**:
- 默认使用人类可读格式（GB, MB等）
- 默认检查当前目录
- 执行超时10秒
- 返回完整的 `df` 命令输出

## 网络工具命令 (net-)

### net-ping - 测试网络连通性

测试到指定主机的网络连通性。

**函数定义**:
```json
{
  "name": "net-ping",
  "description": "测试网络连通性",
  "parameters": [
    {
      "name": "host",
      "description": "要ping的主机名或IP地址", 
      "type": "string",
      "required": true
    },
    {
      "name": "count",
      "description": "ping包数量，默认为4，最大为10",
      "type": "number",
      "required": false
    },
    {
      "name": "timeout",
      "description": "超时时间（秒），默认为10，最大为30",
      "type": "number",
      "required": false
    }
  ]
}
```

**使用示例**:
```json
{
  "name": "net-ping",
  "arguments": "{\"host\": \"example.com\", \"count\": 4, \"timeout\": 10}"
}
```

**返回结果**:
```json
{
  "name": "net-ping",
  "result": {
    "host": "example.com",
    "count": 4,
    "timeout_seconds": 10,
    "success": true,
    "raw_output": "PING example.com (93.184.216.34): 56 data bytes\n64 bytes from 93.184.216.34: icmp_seq=0 ttl=52 time=15.2 ms\n...\n",
    "stats": {
      "summary": "4 packets transmitted, 4 received, 0% packet loss, time 3005ms"
    }
  },
  "error": null
}
```

**注意事项**:
- 主机名格式验证（防止注入攻击）
- ping包数量限制：1-10个
- 超时限制：最大30秒
- 解析并返回统计信息
- 执行超时为 `timeout + 5` 秒

## 错误处理

所有命令调用都遵循统一的错误处理模式：

### 成功响应
```json
{
  "name": "command-name",
  "result": {
    // 命令特定的结果数据
  },
  "error": null
}
```

### 错误响应
```json
{
  "name": "command-name", 
  "result": null,
  "error": "错误描述信息"
}
```

### 常见错误类型
- **参数错误**: 缺少必需参数或参数格式错误
- **权限错误**: 没有足够权限执行操作
- **路径错误**: 路径不存在或被拒绝访问
- **超时错误**: 命令执行超时
- **网络错误**: 网络连接失败或主机不可达
- **系统错误**: 底层系统命令执行失败

## 使用示例

### 完整的函数调用流程

1. **初始化全局注册表**:
```rust
use orion_ai::GlobalFunctionRegistry;

// 初始化（应用启动时调用一次）
GlobalFunctionRegistry::initialize()?;
```

2. **获取注册表并执行命令**:
```rust
use orion_ai::{FunctionCall, FunctionCallInfo};
use serde_json::json;

// 创建函数调用
let function_call = FunctionCall {
    index: Some(0),
    id: "call_001".to_string(),
    r#type: "function".to_string(),
    function: FunctionCallInfo {
        name: "fs-ls".to_string(),
        arguments: "{\"path\": \"/tmp\"}".to_string(),
    },
};

// 执行函数调用
let registry = GlobalFunctionRegistry::get_registry()?;
let result = registry.execute_function(&function_call).await?;

// 处理结果
if result.error.is_none() {
    println!("执行成功: {:?}", result.result);
} else {
    println!("执行失败: {}", result.error.unwrap());
}
```

### 按工具列表过滤

```rust
// 只获取文件系统工具
let tools = vec![
    "fs-ls".to_string(),
    "fs-pwd".to_string(), 
    "fs-cat".to_string(),
    "fs-find".to_string(),
];

let filtered_registry = GlobalFunctionRegistry::get_registry_with_tools(&tools)?;
```

## 最佳实践

### 1. 安全使用
- 始终验证用户提供的路径参数
- 避免在敏感目录执行操作
- 设置合理的超时时间
- 监控命令执行状态

### 2. 性能优化  
- 避免频繁调用系统命令
- 使用缓存机制减少重复查询
- 合理设置超时时间平衡响应速度和可靠性

### 3. 错误处理
- 检查所有命令调用的返回结果
- 提供用户友好的错误信息
- 实现适当的重试机制
- 记录执行日志用于调试

### 4. 扩展开发
- 遵循现有的执行器模式
- 实现完整的安全验证
- 添加充分的单元测试
- 更新文档和使用示例

## 限制说明

### 路径限制
- 不支持相对路径中的 `../`
- 不支持 `~` 家目录快捷方式
- 路径长度限制253字符
- 禁止包含特殊字符

### 执行限制
- 所有命令都有超时限制
- ping命令包数量限制10个
- 文件读取有隐含大小限制
- 进程查询有隐含结果限制

### 安全限制
- 禁止写入操作（如rm, mv, cp）
- 禁止系统修改操作
- 网络访问有目标限制
- 禁止shell命令执行

## 故障排除

### 常见问题及解决方案

#### 问题1: "Path traversal not allowed"
**原因**: 路径包含 `../` 或 `~`
**解决**: 使用绝对路径或相对当前目录的路径

#### 问题2: "Command execution timeout"  
**原因**: 命令执行超时
**解决**: 增加timeout参数或检查目标系统状态

#### 问题3: "Invalid host" 
**原因**: 主机名格式错误或包含非法字符
**解决**: 验证主机名格式，移除特殊字符

#### 问题4: 权限被拒绝
**原因**: 没有足够权限访问文件或目录
**解决**: 检查文件权限或使用有权限的用户执行

### 调试技巧

1. **启用详细日志**:
```rust
env_logger::init();
```

2. **测试单个命令**:
```bash
# 测试基础命令是否可用
ls -la /tmp
uname -a
ping -c 1 example.com
```

3. **检查参数格式**:
```json
// 验证JSON格式正确性
{"path": "/tmp", "pattern": "*.log"}
```

4. **监控资源使用**:
```bash
# 监控命令执行时的系统资源
top
htop
```

## 版本历史

### v1.0.0 (当前版本)
- 初始版本发布
- 支持基础文件系统操作
- 支持系统信息查询
- 支持网络连通性测试
- 完整的安全验证机制

## 未来规划

### 计划功能
- 文件内容搜索（类似 grep）
- 网络连接状态检查（netstat/ss）
- 系统资源监控（内存、CPU使用率）
- 文件权限查询
- 更详细的网络诊断工具

### 改进方向
- 增强缓存机制
- 提供异步批量操作
- 添加配置选项
- 改进错误信息
- 增加更多平台支持

---

本文档将随着系统命令功能的更新而持续完善。如有问题或建议，请提交反馈。