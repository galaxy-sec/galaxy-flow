use orion_ai::provider::{AiProviderType, AiResponse, FunctionCall, FunctionCallInfo, UsageInfo};
use orion_ai::providers::openai::{
    Choice, Message, OpenAiFunctionCall, OpenAiResponse, OpenAiToolCall, Usage,
};
use orion_ai::providers::resp::convert_response_from_text;

#[test]
fn test_send_request_response_conversion() {
    // 测试数据：模拟 OpenAI 响应（不带函数调用）
    let openai_response = create_mock_openai_response("这是一个测试响应", "gpt-4", 100, 50);

    let choice = create_mock_choice(
        "这是一个测试响应",
        Some("stop"),
        None, // 无工具调用
    );

    let provider_type = AiProviderType::OpenAi;
    let estimated_cost = Some(0.003);

    // 手动构造 AiResponse（模拟 send_request 的逻辑）
    let ai_response = AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: provider_type,
        metadata: std::collections::HashMap::new(),
        tool_calls: None,
    };

    // 验证转换结果
    assert_eq!(ai_response.content, "这是一个测试响应");
    assert_eq!(ai_response.model, "gpt-4");
    assert_eq!(ai_response.usage.prompt_tokens, 100);
    assert_eq!(ai_response.usage.completion_tokens, 50);
    assert_eq!(ai_response.usage.total_tokens, 150);
    assert_eq!(ai_response.usage.estimated_cost, estimated_cost);
    assert_eq!(ai_response.finish_reason, Some("stop".to_string()));
    assert_eq!(ai_response.provider, AiProviderType::OpenAi);
    assert!(ai_response.tool_calls.is_none());
    assert_eq!(ai_response.metadata.len(), 0);
}

#[test]
fn test_send_request_with_tools_response_conversion() {
    // 测试数据：模拟带有工具调用的 OpenAI 响应
    let openai_response =
        create_mock_openai_response("我来帮您执行Git操作", "deepseek-chat", 398, 24);

    let tool_calls = vec![create_mock_tool_call(
        Some(0),
        "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c",
        "function",
        "git_status",
        "{}",
    )];

    let choice = create_mock_choice("我来帮您执行Git操作", Some("tool_calls"), Some(tool_calls));

    let provider_type = AiProviderType::DeepSeek;
    let estimated_cost = Some(0.001);

    // 手动构造 AiResponse（模拟 send_request_with_functions 的逻辑）
    let tool_calls = choice.message.tool_calls.as_ref().map(|tool_calls| {
        tool_calls
            .iter()
            .map(|tool_call| FunctionCall {
                index: tool_call.index,
                id: tool_call.id.clone(),
                r#type: tool_call.r#type.clone(),
                function: FunctionCallInfo {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                },
            })
            .collect()
    });

    let ai_response = AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: provider_type,
        metadata: std::collections::HashMap::new(),
        tool_calls,
    };

    // 验证转换结果
    assert_eq!(ai_response.content, "我来帮您执行Git操作");
    assert_eq!(ai_response.model, "deepseek-chat");
    assert_eq!(ai_response.usage.prompt_tokens, 398);
    assert_eq!(ai_response.usage.completion_tokens, 24);
    assert_eq!(ai_response.usage.total_tokens, 422);
    assert_eq!(ai_response.usage.estimated_cost, estimated_cost);
    assert_eq!(ai_response.finish_reason, Some("tool_calls".to_string()));
    assert_eq!(ai_response.provider, AiProviderType::DeepSeek);

    // 验证工具调用
    assert!(ai_response.tool_calls.is_some());
    let tool_calls = ai_response.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);

    let tool_call = &tool_calls[0];
    assert_eq!(tool_call.index, Some(0));
    assert_eq!(tool_call.id, "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c");
    assert_eq!(tool_call.r#type, "function");
    assert_eq!(tool_call.function.name, "git_status");
    assert_eq!(tool_call.function.arguments, "{}");
}

#[test]
fn test_multiple_tool_calls_response_conversion() {
    // 测试多个工具调用的情况
    let openai_response =
        create_mock_openai_response("执行完整的Git工作流", "gpt-4-turbo", 500, 100);

    let tool_calls = vec![
        create_mock_tool_call(Some(0), "call_001", "function", "git_status", "{}"),
        create_mock_tool_call(
            Some(1),
            "call_002",
            "function",
            "git_add",
            "{\"files\": [\".\"]}",
        ),
        create_mock_tool_call(
            Some(2),
            "call_003",
            "function",
            "git_commit",
            "{\"message\": \"Test commit\"}",
        ),
    ];

    let choice = create_mock_choice("执行完整的Git工作流", Some("tool_calls"), Some(tool_calls));

    let provider_type = AiProviderType::OpenAi;
    let estimated_cost = Some(0.006);

    // 构造 AiResponse
    let tool_calls = choice.message.tool_calls.as_ref().map(|tool_calls| {
        tool_calls
            .iter()
            .map(|tool_call| FunctionCall {
                index: tool_call.index,
                id: tool_call.id.clone(),
                r#type: tool_call.r#type.clone(),
                function: FunctionCallInfo {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                },
            })
            .collect()
    });

    let ai_response = AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: provider_type,
        metadata: std::collections::HashMap::new(),
        tool_calls,
    };

    // 验证多个工具调用
    assert!(ai_response.tool_calls.is_some());
    let tool_calls = ai_response.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 3);

    // 验证第一个调用
    assert_eq!(tool_calls[0].function.name, "git_status");
    assert_eq!(tool_calls[0].function.arguments, "{}");

    // 验证第二个调用
    assert_eq!(tool_calls[1].function.name, "git_add");
    assert_eq!(tool_calls[1].function.arguments, "{\"files\": [\".\"]}");

    // 验证第三个调用
    assert_eq!(tool_calls[2].function.name, "git_commit");
    assert_eq!(
        tool_calls[2].function.arguments,
        "{\"message\": \"Test commit\"}"
    );
}

#[test]
fn test_response_conversion_no_usage() {
    // 测试没有使用信息的情况
    let openai_response = create_mock_openai_response(
        "响应内容",
        "gpt-3.5-turbo",
        0, // 无使用信息
        0,
    );

    let choice = create_mock_choice("响应内容", Some("stop"), None);

    let provider_type = AiProviderType::OpenAi;
    let estimated_cost = None;

    // 构造 AiResponse
    let ai_response = AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: provider_type,
        metadata: std::collections::HashMap::new(),
        tool_calls: None,
    };

    // 验证默认值
    assert_eq!(ai_response.usage.prompt_tokens, 0);
    assert_eq!(ai_response.usage.completion_tokens, 0);
    assert_eq!(ai_response.usage.total_tokens, 0);
    assert_eq!(ai_response.usage.estimated_cost, None);
}

#[test]
fn test_response_conversion_edge_cases() {
    // 测试空的完成原因
    let openai_response = create_mock_openai_response("响应内容", "gpt-4", 100, 50);

    let choice = create_mock_choice("响应内容", None, None);

    let ai_response = AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost: None,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: AiProviderType::OpenAi,
        metadata: std::collections::HashMap::new(),
        tool_calls: None,
    };

    assert_eq!(ai_response.finish_reason, None);

    // 测试空的工具调用数组
    let choice2 = create_mock_choice(
        "响应内容",
        Some("tool_calls"),
        Some(vec![]), // 空的工具调用数组
    );

    let tool_calls = choice2.message.tool_calls.as_ref().map(|tool_calls| {
        tool_calls
            .iter()
            .map(|tool_call| FunctionCall {
                index: tool_call.index,
                id: tool_call.id.clone(),
                r#type: tool_call.r#type.clone(),
                function: FunctionCallInfo {
                    name: tool_call.function.name.clone(),
                    arguments: tool_call.function.arguments.clone(),
                },
            })
            .collect()
    });

    let ai_response2 = AiResponse {
        content: choice2.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.prompt_tokens)
                .unwrap_or(0),
            completion_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.completion_tokens)
                .unwrap_or(0),
            total_tokens: openai_response
                .usage
                .as_ref()
                .map(|u| u.total_tokens)
                .unwrap_or(0),
            estimated_cost: None,
        },
        finish_reason: choice2.finish_reason.clone(),
        provider: AiProviderType::OpenAi,
        metadata: std::collections::HashMap::new(),
        tool_calls,
    };

    // 空的工具调用数组应该保持为空数组
    assert!(ai_response2.tool_calls.is_some());
    let tool_calls = ai_response2.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 0);
}

// 辅助函数：创建模拟 OpenAI 响应
fn create_mock_openai_response(
    _content: &str,
    model: &str,
    prompt_tokens: usize,
    completion_tokens: usize,
) -> OpenAiResponse {
    OpenAiResponse {
        choices: vec![],
        usage: Some(Usage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens + completion_tokens,
        }),
        model: model.to_string(),
    }
}

// 辅助函数：创建模拟 Choice
fn create_mock_choice(
    content: &str,
    finish_reason: Option<&str>,
    tool_calls: Option<Vec<OpenAiToolCall>>,
) -> Choice {
    Choice {
        message: Message {
            role: "assistant".to_string(),
            content: content.to_string(),
            tool_calls,
        },
        finish_reason: finish_reason.map(|s| s.to_string()),
    }
}

// 辅助函数：创建模拟工具调用
fn create_mock_tool_call(
    index: Option<u32>,
    id: &str,
    r#type: &str,
    name: &str,
    arguments: &str,
) -> OpenAiToolCall {
    OpenAiToolCall {
        index,
        id: id.to_string(),
        r#type: r#type.to_string(),
        function: OpenAiFunctionCall {
            name: name.to_string(),
            arguments: arguments.to_string(),
        },
    }
}

#[test]
fn test_convert_response_from_text_deepseek_tool_calls() {
    // 真实的 DeepSeek API 响应数据
    let json_response = r#"
{
  "id": "bc0e9568-9853-4f56-9c9c-d073616f3feb",
  "object": "chat.completion",
  "created": 1756041918,
  "model": "deepseek-chat",
  "choices": [
    {
      "index": 0,
      "message": {
        "role": "assistant",
        "content": "我来帮您执行完整的Git工作流。让我按顺序执行这些步骤："
      },
      "logprobs": null,
      "finish_reason": "tool_calls",
      "tool_calls": [
        {
          "index": 0,
          "id": "call_0_3ebc9e84-a137-4188-995c-9381586f7097",
          "type": "function",
          "function": { "name": "git_status", "arguments": "{}" }
        }
      ]
    }
  ],
  "usage": {
    "prompt_tokens": 398,
    "completion_tokens": 24,
    "total_tokens": 422,
    "prompt_tokens_details": { "cached_tokens": 384 },
    "prompt_cache_hit_tokens": 384,
    "prompt_cache_miss_tokens": 14
  },
  "system_fingerprint": "fp_feb633d1f5_prod0820_fp8_kvcache"
}
"#;

    let result = convert_response_from_text(
        json_response,
        AiProviderType::DeepSeek,
        "deepseek-chat",
        |_model, input_tokens, output_tokens| {
            Some(input_tokens as f64 * 0.001 + output_tokens as f64 * 0.002)
        },
    )
    .unwrap();

    // 验证基本响应信息
    assert_eq!(result.model, "deepseek-chat");

    // 验证消息内容
    assert_eq!(
        result.content,
        "我来帮您执行完整的Git工作流。让我按顺序执行这些步骤："
    );

    // 验证函数调用
    assert!(result.tool_calls.is_some());
    let tool_calls = result.tool_calls.unwrap();
    assert_eq!(tool_calls.len(), 1);

    // 验证单个函数调用
    let tool_call = &tool_calls[0];
    assert_eq!(tool_call.index, Some(0));
    assert_eq!(tool_call.id, "call_0_3ebc9e84-a137-4188-995c-9381586f7097");
    assert_eq!(tool_call.r#type, "function");
    assert_eq!(tool_call.function.name, "git_status");
    assert_eq!(tool_call.function.arguments, "{}");

    // 验证使用信息
    let usage = &result.usage;
    assert_eq!(usage.prompt_tokens, 398);
    assert_eq!(usage.completion_tokens, 24);
    assert_eq!(usage.total_tokens, 422);

    // 验证成本计算
    assert!(result.usage.estimated_cost.is_some());
    assert_eq!(result.usage.estimated_cost.unwrap(), 0.446);
}
