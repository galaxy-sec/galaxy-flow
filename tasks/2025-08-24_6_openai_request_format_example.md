# OpenAI Function Calling 请求格式示例

**文件**: `galaxy-flow/tasks/2025-08-24_6_openai_request_format_example.md`  
**创建时间**: 2025-08-24  
**状态**: ✅ 已完成

## 📋 完整的 OpenAI Function Calling 请求格式

### 🎯 标准请求结构

```json
{
  "model": "gpt-4-turbo-preview",
  "messages": [
    {
      "role": "system",
      "content": "你是一个Git助手。当用户要求检查Git状态时，你必须调用git_status函数。"
    },
    {
      "role": "user",
      "content": "请检查当前Git仓库的状态，看看有哪些文件被修改了"
    }
  ],
  "max_tokens": 4096,
  "temperature": 0.1,
  "top_p": 1.0,
  "stream": false,
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "git_status",
        "description": "获取Git仓库状态",
        "parameters": {
          "type": "object",
          "properties": {
            "path": {
              "description": "仓库路径，默认为当前目录",
              "type": "string"
            }
          },
          "required": []
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_add",
        "description": "添加文件到Git暂存区",
        "parameters": {
          "type": "object",
          "properties": {
            "files": {
              "description": "要添加的文件列表，支持通配符",
              "type": "array"
            }
          },
          "required": [
            "files"
          ]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_commit",
        "description": "创建Git提交",
        "parameters": {
          "type": "object",
          "properties": {
            "message": {
              "description": "提交消息",
              "type": "string"
            }
          },
          "required": [
            "message"
          ]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_push",
        "description": "推送提交到远程仓库",
        "parameters": {
          "type": "object",
          "properties": {
            "branch": {
              "description": "分支名称，默认为当前分支",
              "type": "string"
            },
            "remote": {
              "description": "远程仓库名称，默认为origin",
              "type": "string"
            }
          },
          "required": []
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

## 🔧 字段详细说明

### 1. **基本字段**

| 字段 | 类型 | 必需 | 说明 | 示例值 |
|------|------|------|------|---------|
| `model` | string | ✅ | 使用的模型名称 | `"gpt-4-turbo-preview"` |
| `max_tokens` | integer | ✅ | 响应的最大 token 数 | `4096` |
| `temperature` | number | ✅ | 采样温度（0-2） | `0.1` |
| `top_p` | number | ❌ | 核采样（0-1） | `1.0` |
| `stream` | boolean | ✅ | 是否流式响应 | `false` |

### 2. **Messages 字段**

```json
"messages": [
  {
    "role": "system",      // 系统角色
    "content": "你是一个Git助手..."
  },
  {
    "role": "user",       // 用户角色
    "content": "请检查Git仓库的状态..."
  },
  {
    "role": "assistant",  // 助手角色
    "content": "我来帮您检查Git状态..."
  },
  {
    "role": "tool",       // 工具响应角色（可选）
    "tool_call_id": "call_abc123",
    "content": "{\"status\": \"clean\", \"branch\": \"main\"}"
  }
]
```

**角色说明**:
- `system`: 系统指令，设置助手行为
- `user`: 用户输入，包含请求或问题
- `assistant`: 助手响应，可能包含工具调用
- `tool`: 工具执行结果，用于多轮对话

### 3. **Tools 字段**

```json
"tools": [
  {
    "type": "function",                    // 固定值
    "function": {
      "name": "function_name",           // 函数名称（唯一）
      "description": "函数描述...",       // 函数功能说明
      "parameters": {                     // JSON Schema 参数定义
        "type": "object",
        "properties": {
          "param_name": {
            "type": "string",
            "description": "参数描述..."
          }
        },
        "required": ["param_name"]      // 必需参数数组
      }
    }
  }
]
```

### 4. **Tool Choice 字段**

```json
"tool_choice": "auto"  // 可选值："auto"、"none"、"required"或具体函数名
```

**选项说明**:
- `"auto"`: 自动决定是否调用函数（推荐）
- `"none"`: 禁止调用函数
- `"required"`: 必须调用函数
- `"function_name"`: 必须调用指定函数

## 🎯 最佳实践

### 1. **参数设置**

```json
{
  // ✅ 推荐设置（适用于函数调用）
  "temperature": 0.1,           // 低温度提高确定性
  "max_tokens": 4096,           // 足够的响应空间
  "top_p": 1.0,                // 禁用核采样
  "stream": false,              // 禁用流式响应（函数调用）
  "tool_choice": "auto"         // 自动决策
}
```

### 2. **函数定义规范**

```json
{
  "type": "function",
  "function": {
    "name": "get_weather",           // ✅ 使用下划线命名
    "description": "获取指定城市的天气信息",  // ✅ 清晰的功能描述
    "parameters": {
      "type": "object",
      "properties": {
        "location": {
          "type": "string",
          "description": "城市名称，例如：北京、上海、广州"  // ✅ 详细的参数说明
        },
        "units": {
          "type": "string",
          "enum": ["celsius", "fahrenheit"],  // ✅ 枚举约束
          "description": "温度单位，默认为摄氏度"
        }
      },
      "required": ["location"]       // ✅ 明确必需参数
    }
  }
}
```

### 3. **Prompt 工程**

```json
{
  "messages": [
    {
      "role": "system",
      "content": "你是一个专业的Git助手。你可以使用以下工具：\n\n"
        "1. git_status: 检查Git仓库状态\n"
        "2. git_add: 添加文件到暂存区\n"
        "3. git_commit: 创建提交\n"
        "4. git_push: 推送到远程仓库\n\n"
        "当用户询问Git相关问题时，优先使用相应的工具来获取准确信息。"
    },
    {
      "role": "user",
      "content": "请检查当前Git仓库的状态"
    }
  ]
}
```

## 🚀 响应格式示例

### 1. **函数调用响应**

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "created": 1677652288,
  "model": "gpt-4-turbo-preview",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": null,
        "tool_calls": [
          {
            "id": "call_abc123",
            "type": "function",
            "function": {
              "name": "git_status",
              "arguments": "{\"path\": \".\"}"
            }
          }
        ]
      },
      "finish_reason": "tool_calls"
    }
  ],
  "usage": {
    "prompt_tokens": 82,
    "completion_tokens": 17,
    "total_tokens": 99
  }
}
```

### 2. **普通文本响应**

```json
{
  "id": "chatcmpl-123",
  "object": "chat.completion",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "我来帮您检查当前Git仓库的状态...",
        "tool_calls": null
      },
      "finish_reason": "stop"
    }
  ]
}
```

## 📋 完整示例（Git 工作流）

### 场景：完整的 Git 操作工作流

```json
{
  "model": "gpt-4-turbo-preview",
  "messages": [
    {
      "role": "system",
      "content": "你是一个Git工作流助手。请按顺序执行以下Git操作：\n"
        "1. 检查仓库状态\n"
        "2. 添加所有修改的文件\n"
        "3. 创建提交\n"
        "4. 推送到远程仓库\n\n"
        "使用相应的Git函数工具来完成每个步骤。"
    },
    {
      "role": "user",
      "content": "请执行完整的Git提交工作流"
    }
  ],
  "max_tokens": 4096,
  "temperature": 0.1,
  "stream": false,
  "tools": [
    {
      "type": "function",
      "function": {
        "name": "git_status",
        "description": "获取Git仓库状态，显示修改的文件",
        "parameters": {
          "type": "object",
          "properties": {
            "path": {
              "type": "string",
              "description": "Git仓库路径，默认为当前目录"
            }
          },
          "required": []
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_add",
        "description": "将文件添加到Git暂存区",
        "parameters": {
          "type": "object",
          "properties": {
            "files": {
              "type": "array",
              "description": "要添加的文件列表，支持通配符",
              "items": {
                "type": "string"
              }
            }
          },
          "required": ["files"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_commit",
        "description": "创建Git提交",
        "parameters": {
          "type": "object",
          "properties": {
            "message": {
              "type": "string",
              "description": "提交消息"
            }
          },
          "required": ["message"]
        }
      }
    },
    {
      "type": "function",
      "function": {
        "name": "git_push",
        "description": "推送提交到远程仓库",
        "parameters": {
          "type": "object",
          "properties": {
            "branch": {
              "type": "string",
              "description": "要推送的分支名称，默认为当前分支"
            },
            "remote": {
              "type": "string",
              "description": "远程仓库名称，默认为origin"
            }
          },
          "required": []
        }
      }
    }
  ],
  "tool_choice": "auto"
}
```

## 🔧 调试和验证

### 1. **请求验证清单**

- ✅ `model` 字段存在且有效
- ✅ `messages` 数组非空
- ✅ 每个消息都有 `role` 和 `content`
- ✅ `temperature` 在 0-2 范围内
- ✅ `max_tokens` 为正整数
- ✅ `tools` 数组格式正确
- ✅ 每个工具都有 `type` 和 `function`
- ✅ 每个函数都有 `name`、`description`、`parameters`
- ✅ `parameters` 符合 JSON Schema 格式
- ✅ `tool_choice` 值有效

### 2. **常见错误和解决方案**

| 错误 | 原因 | 解决方案 |
|------|------|----------|
| `400 Bad Request` | JSON 格式错误 | 检查 JSON 语法 |
| `429 Too Many Requests` | API 限制 | 等待或增加间隔 |
| `functions not found` | 工具名称错误 | 检查函数定义 |
| `missing required parameter` | 参数缺失 | 检查 required 数组 |
| `invalid parameter type` | 类型错误 | 检查 JSON Schema |

## 📝 总结

这个 OpenAI Function Calling 请求格式示例提供了：

- ✅ **完整的结构定义** - 涵盖所有必需字段
- ✅ **最佳实践指导** - 参数设置和函数定义
- ✅ **实际应用示例** - Git 工作流完整演示
- ✅ **调试和验证** - 错误处理和验证清单
- ✅ **响应格式说明** - 函数调用和普通响应

使用这个格式可以确保与 OpenAI API 的完全兼容性，实现可靠的 Function Calling 功能。

**最终建议**: 在实际应用中，建议使用我们实现的 `OpenAiProvider::convert_to_openai_tools()` 函数来生成正确的工具格式，避免手动构建可能出现的格式错误。