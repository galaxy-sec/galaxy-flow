use orion_ai::func::git::create_git_functions;

#[tokio::test]
async fn test_openai_tool_format_generation() {
    use orion_ai::providers::openai::OpenAiProvider;

    // 创建 Git 函数定义
    let git_functions = create_git_functions();

    // 使用 OpenAI provider 转换为 OpenAI 工具格式
    let openai_tools = OpenAiProvider::convert_to_openai_tools(&git_functions);

    // 将结果转换为 JSON 字符串进行验证
    let json_output = serde_json::to_string_pretty(&openai_tools).unwrap();

    println!("生成的 OpenAI 工具格式:");
    println!("{}", json_output);

    // 解析回结构进行验证
    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_output).unwrap();

    // 验证基本结构
    assert_eq!(parsed.len(), 4, "应该生成4个Git函数工具");

    // 验证第一个工具 (git_status) 的结构
    let git_status_tool = &parsed[0];
    assert_eq!(
        git_status_tool["type"], "function",
        "工具类型应该是function"
    );
    assert_eq!(
        git_status_tool["function"]["name"], "git_status",
        "函数名应该是git_status"
    );
    assert_eq!(
        git_status_tool["function"]["description"], "获取Git仓库状态",
        "描述应该正确"
    );

    // 验证参数结构
    let parameters = &git_status_tool["function"]["parameters"];
    assert_eq!(parameters["type"], "object", "参数类型应该是object");

    // 验证 properties
    let properties = &parameters["properties"];
    assert!(properties.is_object(), "properties应该是对象");

    // 验证 path 参数
    if let Some(path_param) = properties.get("path") {
        assert_eq!(path_param["type"], "string", "path参数类型应该是string");
        assert_eq!(
            path_param["description"], "仓库路径，默认为当前目录",
            "path参数描述应该正确"
        );
    }

    // 验证 required 数组
    let required = &parameters["required"];
    assert!(required.is_array(), "required应该是数组");

    // 验证第二个工具 (git_add) 的结构
    let git_add_tool = &parsed[1];
    assert_eq!(
        git_add_tool["function"]["name"], "git_add",
        "第二个函数应该是git_add"
    );

    // 验证 git_add 的 files 参数 (array类型)
    let git_add_parameters = &git_add_tool["function"]["parameters"];
    let git_add_properties = &git_add_parameters["properties"];

    if let Some(files_param) = git_add_properties.get("files") {
        assert_eq!(files_param["type"], "array", "files参数类型应该是array");
        assert_eq!(
            files_param["description"], "要添加的文件列表，支持通配符",
            "files参数描述应该正确"
        );
    }

    // 验证第三个工具 (git_commit) 的结构
    let git_commit_tool = &parsed[2];
    assert_eq!(
        git_commit_tool["function"]["name"], "git_commit",
        "第三个函数应该是git_commit"
    );

    // 验证 git_commit 应该有必需的 message 参数
    let git_commit_required = &git_commit_tool["function"]["parameters"]["required"];
    assert!(
        git_commit_required.is_array(),
        "git_commit的required应该是数组"
    );

    let required_array = git_commit_required.as_array().unwrap();
    assert!(
        required_array.contains(&serde_json::Value::String("message".to_string())),
        "message应该在required数组中"
    );

    // 验证第四个工具 (git_push) 的结构
    let git_push_tool = &parsed[3];
    assert_eq!(
        git_push_tool["function"]["name"], "git_push",
        "第四个函数应该是git_push"
    );

    // 验证 git_push 的可选参数
    let git_push_required = &git_push_tool["function"]["parameters"]["required"];
    let git_push_required_array = git_push_required.as_array().unwrap();
    assert_eq!(git_push_required_array.len(), 0, "git_push应该没有必需参数");

    // 输出预期的正确格式对比
    println!("\n=== 预期的正确格式示例 ===");
    let expected_format = serde_json::json!([
        {
            "type": "function",
            "function": {
                "name": "get_weather",
                "description": "获取指定地点的天气信息",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "地点名称"
                        },
                        "date": {
                            "type": "string",
                            "description": "日期，格式为YYYY-MM-DD"
                        }
                    },
                    "required": ["location"]
                }
            }
        }
    ]);

    println!("预期格式:");
    println!(
        "{}",
        serde_json::to_string_pretty(&expected_format).unwrap()
    );

    // 比较结构格式
    println!("\n=== 格式对比 ===");
    println!("生成的工具数量: {}", parsed.len());
    println!("预期工具数量: 1 (示例)");

    // 验证我们的生成是否与预期格式一致
    for (i, tool) in parsed.iter().enumerate() {
        let function = &tool["function"];
        let parameters = &function["parameters"];

        println!("工具 {}: {}", i + 1, function["name"]);
        println!("  - type: {}", tool["type"]);
        println!("  - parameters.type: {}", parameters["type"]);
        println!(
            "  - parameters.properties: {}",
            if parameters["properties"].is_object() {
                "✅ 对象"
            } else {
                "❌ 非对象"
            }
        );
        println!(
            "  - parameters.required: {}",
            if parameters["required"].is_array() {
                "✅ 数组"
            } else {
                "❌ 非数组"
            }
        );

        // 验证必需字段是否存在
        assert!(tool.get("type").is_some(), "每个工具都应该有type字段");
        assert!(
            tool.get("function").is_some(),
            "每个工具都应该有function字段"
        );
        assert!(function.get("name").is_some(), "每个函数都应该有name字段");
        assert!(
            function.get("description").is_some(),
            "每个函数都应该有description字段"
        );
        assert!(parameters.get("type").is_some(), "每个参数都应该有type字段");
        assert!(
            parameters.get("properties").is_some(),
            "每个参数都应该有properties字段"
        );
        assert!(
            parameters.get("required").is_some(),
            "每个参数都应该有required字段"
        );
    }
}

#[tokio::test]
async fn test_openai_tool_parameter_type_mapping() {
    use orion_ai::providers::openai::OpenAiProvider;

    // 测试各种参数类型的映射
    let test_functions = vec![
        orion_ai::provider::FunctionDefinition {
            name: "test_string".to_string(),
            description: "测试字符串参数".to_string(),
            parameters: vec![orion_ai::provider::FunctionParameter {
                name: "string_param".to_string(),
                description: "字符串参数".to_string(),
                r#type: "string".to_string(),
                required: true,
            }],
        },
        orion_ai::provider::FunctionDefinition {
            name: "test_array".to_string(),
            description: "测试数组参数".to_string(),
            parameters: vec![orion_ai::provider::FunctionParameter {
                name: "array_param".to_string(),
                description: "数组参数".to_string(),
                r#type: "array".to_string(),
                required: false,
            }],
        },
        orion_ai::provider::FunctionDefinition {
            name: "test_number".to_string(),
            description: "测试数字参数".to_string(),
            parameters: vec![orion_ai::provider::FunctionParameter {
                name: "number_param".to_string(),
                description: "数字参数".to_string(),
                r#type: "number".to_string(),
                required: false,
            }],
        },
        orion_ai::provider::FunctionDefinition {
            name: "test_boolean".to_string(),
            description: "测试布尔参数".to_string(),
            parameters: vec![orion_ai::provider::FunctionParameter {
                name: "boolean_param".to_string(),
                description: "布尔参数".to_string(),
                r#type: "boolean".to_string(),
                required: false,
            }],
        },
    ];

    let openai_tools = OpenAiProvider::convert_to_openai_tools(&test_functions);
    let json_output = serde_json::to_string_pretty(&openai_tools).unwrap();

    println!("参数类型映射测试:");
    println!("{}", json_output);

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_output).unwrap();

    // 验证字符串类型映射
    let string_tool = &parsed[0];
    assert_eq!(
        string_tool["function"]["parameters"]["properties"]["string_param"]["type"],
        "string"
    );

    // 验证数组类型映射
    let array_tool = &parsed[1];
    assert_eq!(
        array_tool["function"]["parameters"]["properties"]["array_param"]["type"],
        "array"
    );

    // 验证数字类型映射
    let number_tool = &parsed[2];
    assert_eq!(
        number_tool["function"]["parameters"]["properties"]["number_param"]["type"],
        "number"
    );

    // 验证布尔类型映射
    let boolean_tool = &parsed[3];
    assert_eq!(
        boolean_tool["function"]["parameters"]["properties"]["boolean_param"]["type"],
        "boolean"
    );
}

#[tokio::test]
async fn test_openai_tool_required_parameters() {
    use orion_ai::providers::openai::OpenAiProvider;

    // 测试必需参数和可选参数
    let test_function = orion_ai::provider::FunctionDefinition {
        name: "test_required".to_string(),
        description: "测试必需参数".to_string(),
        parameters: vec![
            orion_ai::provider::FunctionParameter {
                name: "required_param".to_string(),
                description: "必需参数".to_string(),
                r#type: "string".to_string(),
                required: true,
            },
            orion_ai::provider::FunctionParameter {
                name: "optional_param".to_string(),
                description: "可选参数".to_string(),
                r#type: "string".to_string(),
                required: false,
            },
        ],
    };

    let openai_tools = OpenAiProvider::convert_to_openai_tools(&[test_function]);
    let json_output = serde_json::to_string_pretty(&openai_tools).unwrap();

    println!("必需参数测试:");
    println!("{}", json_output);

    let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_output).unwrap();
    let tool = &parsed[0];
    let required = &tool["function"]["parameters"]["required"];

    // 验证 required 是数组
    assert!(required.is_array());

    let required_array = required.as_array().unwrap();
    assert_eq!(required_array.len(), 1, "应该只有1个必需参数");
    assert!(
        required_array.contains(&serde_json::Value::String("required_param".to_string())),
        "应该包含required_param"
    );
    assert!(
        !required_array.contains(&serde_json::Value::String("optional_param".to_string())),
        "不应该包含optional_param"
    );
}
