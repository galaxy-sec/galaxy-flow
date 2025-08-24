//! 响应转换器模块
//!
//! 这个模块负责将各种 AI 提供商的响应转换为统一的 AiResponse 格式
//! 主要包含从 OpenAI 格式响应到 AiResponse 的转换逻辑

use crate::error::{AiErrReason, AiResult};
use crate::provider::{AiProviderType, AiResponse, FunctionCall, FunctionCallInfo, UsageInfo};
use crate::providers::openai::{Choice, OpenAiResponse};
use orion_error::{ToStructError, UvsBizFrom, UvsConfFrom, UvsReason};

/// OpenAI 响应转换器
pub struct OpenAiResponseConverter {
    provider_type: AiProviderType,
}

impl OpenAiResponseConverter {
    /// 创建新的转换器实例
    pub fn new(provider_type: AiProviderType) -> Self {
        Self { provider_type }
    }

    /// 转换 OpenAI 响应到 AiResponse（不带函数调用）
    ///
    /// 这个方法对应于 `send_request` 中的响应转换逻辑
    pub fn convert_response(
        &self,
        openai_response: OpenAiResponse,
        request_model: &str,
        cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
    ) -> AiResponse {
        let choice = openai_response
            .choices
            .first()
            .expect("No choices in response");

        AiResponse {
            content: choice.message.content.clone(),
            model: openai_response.model.clone(),
            usage: self.convert_usage(&openai_response, request_model, cost_calculator),
            finish_reason: choice.finish_reason.clone(),
            provider: self.provider_type,
            metadata: std::collections::HashMap::new(),
            tool_calls: None,
        }
    }

    /// 转换 OpenAI 响应到 AiResponse（带函数调用）
    ///
    /// 这个方法对应于 `send_request_with_functions` 中的响应转换逻辑
    pub fn convert_response_with_functions(
        &self,
        openai_response: OpenAiResponse,
        request_model: &str,
        cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
    ) -> AiResponse {
        let choice = openai_response
            .choices
            .first()
            .expect("No choices in response");

        let tool_calls = choice.tool_calls.as_ref().map(|tool_calls| {
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

        AiResponse {
            content: choice.message.content.clone(),
            model: openai_response.model.clone(),
            usage: self.convert_usage(&openai_response, request_model, cost_calculator),
            finish_reason: choice.finish_reason.clone(),
            provider: self.provider_type,
            metadata: std::collections::HashMap::new(),
            tool_calls,
        }
    }

    /// 转换使用信息和成本计算
    fn convert_usage(
        &self,
        openai_response: &OpenAiResponse,
        request_model: &str,
        cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
    ) -> UsageInfo {
        let prompt_tokens = openai_response
            .usage
            .as_ref()
            .map(|u| u.prompt_tokens)
            .unwrap_or(0);

        let completion_tokens = openai_response
            .usage
            .as_ref()
            .map(|u| u.completion_tokens)
            .unwrap_or(0);

        let total_tokens = openai_response
            .usage
            .as_ref()
            .map(|u| u.total_tokens)
            .unwrap_or(0);

        let estimated_cost = cost_calculator(request_model, prompt_tokens, completion_tokens);

        UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            estimated_cost,
        }
    }
}

/// 独立的响应转换函数
///
/// 这个函数从 send_request_with_functions 中抽出，专门处理带函数调用的响应转换
/// 自动根据响应数据判断是否需要解析函数调用
pub fn convert_response_with_functions_helper(
    openai_response: OpenAiResponse,
    provider_type: AiProviderType,
    request_model: &str,
    cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
) -> AiResult<AiResponse> {
    convert_response_auto(
        openai_response,
        provider_type,
        request_model,
        cost_calculator,
    )
}

/// 独立的响应转换函数
///
/// 这个函数从 send_request 中抽出，专门处理普通文本响应的转换
/// 自动根据响应数据判断是否需要解析函数调用
pub fn convert_response_helper(
    openai_response: OpenAiResponse,
    provider_type: AiProviderType,
    request_model: &str,
    cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
) -> AiResult<AiResponse> {
    convert_response_auto(
        openai_response,
        provider_type,
        request_model,
        cost_calculator,
    )
}

/// 统一的响应转换实现
///
/// 这个函数自动根据响应数据判断是否需要解析函数调用
fn convert_response_auto(
    openai_response: OpenAiResponse,
    provider_type: AiProviderType,
    request_model: &str,
    cost_calculator: impl Fn(&str, usize, usize) -> Option<f64>,
) -> AiResult<AiResponse> {
    let choice = openai_response.choices.first().ok_or_else(|| {
        AiErrReason::Uvs(UvsReason::from_biz(
            "TODO: no choices in response".to_string(),
        ))
        .to_err()
    })?;

    // 自动判断是否需要解析函数调用
    let tool_calls = choice.tool_calls.as_ref().map(|tool_calls| {
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

    // 转换使用信息
    let prompt_tokens = openai_response
        .usage
        .as_ref()
        .map(|u| u.prompt_tokens)
        .unwrap_or(0);

    let completion_tokens = openai_response
        .usage
        .as_ref()
        .map(|u| u.completion_tokens)
        .unwrap_or(0);

    let total_tokens = openai_response
        .usage
        .as_ref()
        .map(|u| u.total_tokens)
        .unwrap_or(0);

    let estimated_cost = cost_calculator(request_model, prompt_tokens, completion_tokens);

    Ok(AiResponse {
        content: choice.message.content.clone(),
        model: openai_response.model.clone(),
        usage: UsageInfo {
            prompt_tokens,
            completion_tokens,
            total_tokens,
            estimated_cost,
        },
        finish_reason: choice.finish_reason.clone(),
        provider: provider_type,
        metadata: std::collections::HashMap::new(),
        tool_calls,
    })
}

#[cfg(test)]
mod helper_tests {
    use super::*;
    use crate::providers::openai::{Message, OpenAiFunctionCall, OpenAiToolCall, Usage};

    #[test]
    fn test_convert_response_with_functions_helper_success() {
        // 创建模拟的 OpenAI 响应（带函数调用）
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "我来帮您执行Git操作".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![OpenAiToolCall {
                    index: Some(0),
                    id: "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c".to_string(),
                    r#type: "function".to_string(),
                    function: OpenAiFunctionCall {
                        name: "git_status".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
            }],
            usage: Some(Usage {
                prompt_tokens: 398,
                completion_tokens: 24,
                total_tokens: 422,
            }),
            model: "deepseek-chat".to_string(),
        };

        // 转换响应
        let result = convert_response_with_functions_helper(
            openai_response,
            AiProviderType::DeepSeek,
            "deepseek-chat",
            |_, _, _| Some(0.001),
        );

        // 验证结果
        assert!(result.is_ok());
        let response = result.unwrap();

        // 验证基本响应信息
        assert_eq!(response.content, "我来帮您执行Git操作");
        assert_eq!(response.model, "deepseek-chat");
        assert_eq!(response.usage.prompt_tokens, 398);
        assert_eq!(response.usage.completion_tokens, 24);
        assert_eq!(response.usage.total_tokens, 422);
        assert_eq!(response.usage.estimated_cost, Some(0.001));
        assert_eq!(response.finish_reason, Some("tool_calls".to_string()));
        assert_eq!(response.provider, AiProviderType::DeepSeek);

        // 验证函数调用
        assert!(response.tool_calls.is_some());
        let tool_calls = response.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);

        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.index, Some(0));
        assert_eq!(tool_call.id, "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c");
        assert_eq!(tool_call.r#type, "function");
        assert_eq!(tool_call.function.name, "git_status");
        assert_eq!(tool_call.function.arguments, "{}");
    }

    #[test]
    fn test_convert_response_with_functions_helper_no_choices() {
        // 创建没有 choices 的响应
        let openai_response = OpenAiResponse {
            choices: vec![], // 空数组
            usage: None,
            model: "test-model".to_string(),
        };

        let result = convert_response_with_functions_helper(
            openai_response,
            AiProviderType::OpenAi,
            "test-model",
            |_, _, _| None,
        );

        // 验证返回错误
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            "[800] business logic error << \"TODO: no choices in response\""
        );
    }

    #[test]
    fn test_convert_response_helper_success() {
        // 创建模拟的 OpenAI 响应（无函数调用）
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "这是一个测试响应".to_string(),
                },
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".to_string(),
        };

        // 转换响应
        let result = convert_response_helper(
            openai_response,
            AiProviderType::OpenAi,
            "gpt-4",
            |_, _, _| Some(0.003),
        );

        // 验证结果
        assert!(result.is_ok());
        let response = result.unwrap();

        // 验证响应信息
        assert_eq!(response.content, "这是一个测试响应");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.usage.prompt_tokens, 100);
        assert_eq!(response.usage.completion_tokens, 50);
        assert_eq!(response.usage.total_tokens, 150);
        assert_eq!(response.usage.estimated_cost, Some(0.003));
        assert_eq!(response.finish_reason, Some("stop".to_string()));
        assert_eq!(response.provider, AiProviderType::OpenAi);
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_convert_response_helper_no_choices() {
        // 创建没有 choices 的响应
        let openai_response = OpenAiResponse {
            choices: vec![], // 空数组
            usage: None,
            model: "test-model".to_string(),
        };

        let result = convert_response_helper(
            openai_response,
            AiProviderType::OpenAi,
            "test-model",
            |_, _, _| None,
        );

        // 验证返回错误
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert_eq!(
            error.to_string(),
            "[800] business logic error << \"TODO: no choices in response\""
        );
    }

    #[test]
    fn test_convert_response_helper_no_usage() {
        // 创建没有使用信息的响应
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "响应内容".to_string(),
                },
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: None, // 无使用信息
            model: "gpt-3.5-turbo".to_string(),
        };

        let result = convert_response_helper(
            openai_response,
            AiProviderType::OpenAi,
            "gpt-3.5-turbo",
            |_, _, _| None,
        );

        // 验证结果
        assert!(result.is_ok());
        let response = result.unwrap();

        // 验证默认值
        assert_eq!(response.content, "响应内容");
        assert_eq!(response.usage.prompt_tokens, 0);
        assert_eq!(response.usage.completion_tokens, 0);
        assert_eq!(response.usage.total_tokens, 0);
        assert_eq!(response.usage.estimated_cost, None);
        assert_eq!(response.finish_reason, Some("stop".to_string()));
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_convert_response_with_functions_helper_multiple_tool_calls() {
        // 创建多个函数调用的响应
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "执行完整的Git工作流".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![
                    OpenAiToolCall {
                        index: Some(0),
                        id: "call_001".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_status".to_string(),
                            arguments: "{}".to_string(),
                        },
                    },
                    OpenAiToolCall {
                        index: Some(1),
                        id: "call_002".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_add".to_string(),
                            arguments: "{\"files\": [\".\"]}".to_string(),
                        },
                    },
                    OpenAiToolCall {
                        index: Some(2),
                        id: "call_003".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_commit".to_string(),
                            arguments: "{\"message\": \"Test commit\"}".to_string(),
                        },
                    },
                ]),
            }],
            usage: Some(Usage {
                prompt_tokens: 500,
                completion_tokens: 100,
                total_tokens: 600,
            }),
            model: "gpt-4-turbo".to_string(),
        };

        let result = convert_response_with_functions_helper(
            openai_response,
            AiProviderType::OpenAi,
            "gpt-4-turbo",
            |_, _, _| Some(0.006),
        );

        // 验证结果
        assert!(result.is_ok());
        let response = result.unwrap();

        // 验证多个函数调用
        assert!(response.tool_calls.is_some());
        let tool_calls = response.tool_calls.as_ref().unwrap();
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
    fn test_convert_response_auto_detect_tool_calls() {
        // 创建带有函数调用的响应
        let openai_response_with_tool_calls = create_openai_response_with_tool_calls();

        // 创建不带有函数调用的响应
        let openai_response_without_tool_calls = create_openai_response_without_tool_calls();

        // 测试 convert_response_with_functions_helper 自动检测
        let result_with_tool_calls = convert_response_with_functions_helper(
            openai_response_with_tool_calls,
            AiProviderType::DeepSeek,
            "deepseek-chat",
            |_, _, _| Some(0.001),
        );

        let result_without_tool_calls = convert_response_with_functions_helper(
            openai_response_without_tool_calls,
            AiProviderType::OpenAi,
            "gpt-4",
            |_, _, _| Some(0.003),
        );

        // 验证有函数调用的响应
        assert!(result_with_tool_calls.is_ok());
        let response_with_tool_calls = result_with_tool_calls.unwrap();
        assert!(response_with_tool_calls.tool_calls.is_some());
        let tool_calls = response_with_tool_calls.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].function.name, "git_status");

        // 验证无函数调用的响应
        assert!(result_without_tool_calls.is_ok());
        let response_without_tool_calls = result_without_tool_calls.unwrap();
        assert!(response_without_tool_calls.tool_calls.is_none());

        // 测试 convert_response_helper 自动检测
        let openai_response_with_tool_calls = create_openai_response_with_tool_calls();
        let openai_response_without_tool_calls = create_openai_response_without_tool_calls();

        let result_helper_with_tool_calls = convert_response_helper(
            openai_response_with_tool_calls,
            AiProviderType::DeepSeek,
            "deepseek-chat",
            |_, _, _| Some(0.001),
        );

        let result_helper_without_tool_calls = convert_response_helper(
            openai_response_without_tool_calls,
            AiProviderType::OpenAi,
            "gpt-4",
            |_, _, _| Some(0.003),
        );

        // 验证 convert_response_helper 也正确检测
        assert!(result_helper_with_tool_calls.is_ok());
        let response_helper_with_tool_calls = result_helper_with_tool_calls.unwrap();
        assert!(response_helper_with_tool_calls.tool_calls.is_some());

        assert!(result_helper_without_tool_calls.is_ok());
        let response_helper_without_tool_calls = result_helper_without_tool_calls.unwrap();
        assert!(response_helper_without_tool_calls.tool_calls.is_none());
    }

    // 辅助函数：创建带有函数调用的响应
    fn create_openai_response_with_tool_calls() -> OpenAiResponse {
        OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "我来帮您执行Git操作".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![OpenAiToolCall {
                    index: Some(0),
                    id: "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c".to_string(),
                    r#type: "function".to_string(),
                    function: OpenAiFunctionCall {
                        name: "git_status".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
            }],
            usage: Some(Usage {
                prompt_tokens: 398,
                completion_tokens: 24,
                total_tokens: 422,
            }),
            model: "deepseek-chat".to_string(),
        }
    }

    // 辅助函数：创建不带有函数调用的响应
    fn create_openai_response_without_tool_calls() -> OpenAiResponse {
        OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "这是一个测试响应".to_string(),
                },
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".to_string(),
        }
    }

    #[test]
    fn test_convert_response_auto_empty_tool_calls() {
        // 创建 tool_calls 为空数组的情况
        let openai_response = create_openai_response_with_empty_tool_calls();

        // 测试两个函数都正确处理空数组
        let result1 = convert_response_with_functions_helper(
            openai_response,
            AiProviderType::OpenAi,
            "gpt-4",
            |_, _, _| Some(0.003),
        );

        // 再次创建相同的响应用于第二个测试
        let openai_response = create_openai_response_with_empty_tool_calls();
        let result2 = convert_response_helper(
            openai_response,
            AiProviderType::OpenAi,
            "gpt-4",
            |_, _, _| Some(0.003),
        );

        // 验证两个结果都包含空的 tool_calls 数组
        assert!(result1.is_ok());
        let response1 = result1.unwrap();
        assert!(response1.tool_calls.is_some());
        assert_eq!(response1.tool_calls.as_ref().unwrap().len(), 0);

        assert!(result2.is_ok());
        let response2 = result2.unwrap();
        assert!(response2.tool_calls.is_some());
        assert_eq!(response2.tool_calls.as_ref().unwrap().len(), 0);
    }

    // 辅助函数：创建带有空函数调用数组的响应
    fn create_openai_response_with_empty_tool_calls() -> OpenAiResponse {
        OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "响应内容".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![]), // 空数组
            }],
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::providers::openai::{Message, OpenAiFunctionCall, OpenAiToolCall, Usage};

    #[test]
    fn test_convert_response_without_functions() {
        let converter = OpenAiResponseConverter::new(AiProviderType::OpenAi);

        // 创建模拟的 OpenAI 响应
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "这是一个测试响应".to_string(),
                },
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".to_string(),
        };

        // 转换响应
        let response = converter.convert_response(openai_response, "gpt-4", |_, _, _| Some(0.003));

        // 验证结果
        assert_eq!(response.content, "这是一个测试响应");
        assert_eq!(response.model, "gpt-4");
        assert_eq!(response.usage.prompt_tokens, 100);
        assert_eq!(response.usage.completion_tokens, 50);
        assert_eq!(response.usage.total_tokens, 150);
        assert_eq!(response.usage.estimated_cost, Some(0.003));
        assert_eq!(response.finish_reason, Some("stop".to_string()));
        assert_eq!(response.provider, AiProviderType::OpenAi);
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_convert_response_with_functions() {
        let converter = OpenAiResponseConverter::new(AiProviderType::DeepSeek);

        // 创建模拟的 OpenAI 响应（带函数调用）
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "我来帮您执行Git操作".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![OpenAiToolCall {
                    index: Some(0),
                    id: "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c".to_string(),
                    r#type: "function".to_string(),
                    function: OpenAiFunctionCall {
                        name: "git_status".to_string(),
                        arguments: "{}".to_string(),
                    },
                }]),
            }],
            usage: Some(Usage {
                prompt_tokens: 398,
                completion_tokens: 24,
                total_tokens: 422,
            }),
            model: "deepseek-chat".to_string(),
        };

        // 转换响应
        let response = converter.convert_response_with_functions(
            openai_response,
            "deepseek-chat",
            |_, _, _| Some(0.001),
        );

        // 验证基本响应信息
        assert_eq!(response.content, "我来帮您执行Git操作");
        assert_eq!(response.model, "deepseek-chat");
        assert_eq!(response.usage.prompt_tokens, 398);
        assert_eq!(response.usage.completion_tokens, 24);
        assert_eq!(response.usage.total_tokens, 422);
        assert_eq!(response.usage.estimated_cost, Some(0.001));
        assert_eq!(response.finish_reason, Some("tool_calls".to_string()));
        assert_eq!(response.provider, AiProviderType::DeepSeek);

        // 验证函数调用
        assert!(response.tool_calls.is_some());
        let tool_calls = response.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 1);

        let tool_call = &tool_calls[0];
        assert_eq!(tool_call.index, Some(0));
        assert_eq!(tool_call.id, "call_0_889decaf-c79e-4e8c-8655-fe0d7805298c");
        assert_eq!(tool_call.r#type, "function");
        assert_eq!(tool_call.function.name, "git_status");
        assert_eq!(tool_call.function.arguments, "{}");
    }

    #[test]
    fn test_convert_response_no_usage() {
        let converter = OpenAiResponseConverter::new(AiProviderType::OpenAi);

        // 创建没有使用信息的响应
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "响应内容".to_string(),
                },
                finish_reason: Some("stop".to_string()),
                tool_calls: None,
            }],
            usage: None,
            model: "gpt-3.5-turbo".to_string(),
        };

        // 转换响应（使用不计算成本的函数）
        let response = converter.convert_response(openai_response, "gpt-3.5-turbo", |_, _, _| None);

        // 验证默认值
        assert_eq!(response.content, "响应内容");
        assert_eq!(response.usage.prompt_tokens, 0);
        assert_eq!(response.usage.completion_tokens, 0);
        assert_eq!(response.usage.total_tokens, 0);
        assert_eq!(response.usage.estimated_cost, None);
        assert_eq!(response.finish_reason, Some("stop".to_string()));
        assert!(response.tool_calls.is_none());
    }

    #[test]
    fn test_convert_response_multiple_tool_calls() {
        let converter = OpenAiResponseConverter::new(AiProviderType::OpenAi);

        // 创建多个函数调用的响应
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "执行完整的Git工作流".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![
                    OpenAiToolCall {
                        index: Some(0),
                        id: "call_001".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_status".to_string(),
                            arguments: "{}".to_string(),
                        },
                    },
                    OpenAiToolCall {
                        index: Some(1),
                        id: "call_002".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_add".to_string(),
                            arguments: "{\"files\": [\".\"]}".to_string(),
                        },
                    },
                    OpenAiToolCall {
                        index: Some(2),
                        id: "call_003".to_string(),
                        r#type: "function".to_string(),
                        function: OpenAiFunctionCall {
                            name: "git_commit".to_string(),
                            arguments: "{\"message\": \"Test commit\"}".to_string(),
                        },
                    },
                ]),
            }],
            usage: Some(Usage {
                prompt_tokens: 500,
                completion_tokens: 100,
                total_tokens: 600,
            }),
            model: "gpt-4-turbo".to_string(),
        };

        let response =
            converter.convert_response_with_functions(openai_response, "gpt-4-turbo", |_, _, _| {
                Some(0.006)
            });

        // 验证多个函数调用
        assert!(response.tool_calls.is_some());
        let tool_calls = response.tool_calls.as_ref().unwrap();
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
    fn test_convert_response_empty_tool_calls() {
        let converter = OpenAiResponseConverter::new(AiProviderType::OpenAi);

        // 创建空的工具调用数组
        let openai_response = OpenAiResponse {
            choices: vec![Choice {
                message: Message {
                    role: "assistant".to_string(),
                    content: "响应内容".to_string(),
                },
                finish_reason: Some("tool_calls".to_string()),
                tool_calls: Some(vec![]), // 空数组
            }],
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            model: "gpt-4".to_string(),
        };

        let response =
            converter
                .convert_response_with_functions(openai_response, "gpt-4", |_, _, _| Some(0.003));

        // 空的工具调用数组应该保持为空数组
        assert!(response.tool_calls.is_some());
        let tool_calls = response.tool_calls.as_ref().unwrap();
        assert_eq!(tool_calls.len(), 0);
    }
}
