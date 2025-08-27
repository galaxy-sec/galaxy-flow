use galaxy_flow::ability::ai_fun::GxAIFun;
use orion_ai::GlobalFunctionRegistry;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 测试工具过滤功能");

    // 初始化全局注册表
    println!("\n📋 初始化全局注册表");
    GlobalFunctionRegistry::initialize()?;
    println!("✅ 全局注册表初始化完成");

    // 测试 1: 验证全局注册表初始化
    println!("\n📋 测试 1: 验证全局注册表初始化");
    let registry = GlobalFunctionRegistry::get_registry()?;
    let all_functions = registry.get_supported_function_names();
    println!("✅ 全局注册表包含 {} 个函数", all_functions.len());
    for func in &all_functions {
        println!("  🔧 {}", func);
    }

    // 测试 2: 测试指定工具列表的过滤
    println!("\n📋 测试 2: 测试指定工具列表的过滤");
    let specific_tools = vec!["git-status".to_string(), "git-add".to_string()];
    let filtered_registry = GlobalFunctionRegistry::get_registry_with_tools(&specific_tools)?;

    println!("✅ 指定工具列表: {:?}", specific_tools);
    println!(
        "✅ 过滤后得到 {} 个函数",
        filtered_registry.get_supported_function_names().len()
    );
    for func in filtered_registry.get_supported_function_names() {
        println!("  🔧 {}", func);
    }

    // 验证过滤结果
    let filtered_functions = filtered_registry.get_supported_function_names();
    assert_eq!(filtered_functions.len(), 2, "应该只过滤出2个函数");
    assert!(
        filtered_functions.contains(&"git-status".to_string()),
        "应该包含git_status"
    );
    assert!(
        filtered_functions.contains(&"git-add".to_string()),
        "应该包含git_add"
    );
    assert!(
        !filtered_functions.contains(&"git-commit".to_string()),
        "不应该包含git_commit"
    );

    // 测试 3: 测试单个工具的过滤
    println!("\n📋 测试 3: 测试单个工具的过滤");
    let single_tool = vec!["git-status".to_string()];
    let single_registry = GlobalFunctionRegistry::get_registry_with_tools(&single_tool)?;

    println!("✅ 单个工具: {:?}", single_tool);
    println!(
        "✅ 过滤后得到 {} 个函数",
        single_registry.get_supported_function_names().len()
    );
    assert_eq!(
        single_registry.get_supported_function_names().len(),
        1,
        "应该只过滤出1个函数"
    );
    assert_eq!(
        single_registry.get_supported_function_names()[0],
        "git-status",
        "应该是git_status"
    );

    // 测试 4: 测试空工具列表（返回所有函数）
    println!("\n📋 测试 4: 测试空工具列表");
    let empty_tools: Vec<String> = vec![];
    let all_filtered_registry = GlobalFunctionRegistry::get_registry_with_tools(&empty_tools)?;

    println!("✅ 空工具列表: {:?}", empty_tools);
    println!(
        "✅ 过滤后得到 {} 个函数",
        all_filtered_registry.get_supported_function_names().len()
    );
    assert_eq!(
        all_filtered_registry.get_supported_function_names().len(),
        all_functions.len(),
        "应该返回所有函数"
    );

    // 测试 5: 测试 GxAIFun 的工具过滤功能
    println!("\n📋 测试 5: 测试 GxAIFun 的工具过滤功能");

    // 创建带有指定工具的 GxAIFun
    let ai_fun_specific = GxAIFun::default()
        .with_enable_function_calling(true)
        .with_role(Some("developer".to_string()))
        .with_task(Some("测试工具过滤".to_string()))
        .with_tools(vec!["git-status".to_string(), "git-add".to_string()]);

    println!("✅ GxAIFun 工具列表: {:?}", ai_fun_specific.tools());
    assert_eq!(ai_fun_specific.tools().len(), 2, "应该有2个指定工具");

    // 创建带有单个工具的 GxAIFun
    let ai_fun_single = GxAIFun::default()
        .with_enable_function_calling(true)
        .with_role(Some("developer".to_string()))
        .with_task(Some("测试单个工具".to_string()))
        .with_tools(vec!["git-status".to_string()]);

    println!("✅ GxAIFun 单个工具: {:?}", ai_fun_single.tools());
    assert_eq!(ai_fun_single.tools().len(), 1, "应该有1个指定工具");

    // 创建带有空工具列表的 GxAIFun（使用所有工具）
    let ai_fun_all = GxAIFun::default()
        .with_enable_function_calling(true)
        .with_role(Some("developer".to_string()))
        .with_task(Some("测试所有工具".to_string()))
        .with_tools(vec![]); // 空列表表示使用所有工具

    println!("✅ GxAIFun 空工具列表: {:?}", ai_fun_all.tools());
    assert_eq!(ai_fun_all.tools().len(), 0, "应该有0个指定工具（使用所有）");

    // 测试 6: 验证 with_tools 方法
    println!("\n📋 测试 6: 验证 with_tools 方法");
    let ai_fun_with_tools = GxAIFun::default()
        .with_enable_function_calling(true)
        .with_role(Some("developer".to_string()))
        .with_task(Some("with_tools 方法测试".to_string()))
        .with_tools(vec!["git-commit".to_string(), "git-push".to_string()]);

    println!("✅ with_tools 工具列表: {:?}", ai_fun_with_tools.tools());
    assert_eq!(ai_fun_with_tools.tools().len(), 2, "应该有2个工具");
    assert!(ai_fun_with_tools
        .tools()
        .contains(&"git-commit".to_string()));
    assert!(ai_fun_with_tools.tools().contains(&"git-push".to_string()));

    // 测试 7: 验证不存在的工具
    println!("\n📋 测试 7: 验证不存在的工具");
    let non_existent_tools = vec!["nonexistent_tool".to_string()];
    let non_existent_registry =
        GlobalFunctionRegistry::get_registry_with_tools(&non_existent_tools)?;

    println!("✅ 不存在工具: {:?}", non_existent_tools);
    println!(
        "✅ 过滤后得到 {} 个函数",
        non_existent_registry.get_supported_function_names().len()
    );
    assert_eq!(
        non_existent_registry.get_supported_function_names().len(),
        0,
        "不存在的工具应该返回空列表"
    );

    // 测试 8: 验证部分存在部分不存在的混合情况
    println!("\n📋 测试 8: 验证混合工具列表");
    let mixed_tools = vec!["git-status".to_string(), "nonexistent_tool".to_string()];
    let mixed_registry = GlobalFunctionRegistry::get_registry_with_tools(&mixed_tools)?;

    println!("✅ 混合工具列表: {:?}", mixed_tools);
    println!(
        "✅ 过滤后得到 {} 个函数",
        mixed_registry.get_supported_function_names().len()
    );
    assert_eq!(
        mixed_registry.get_supported_function_names().len(),
        1,
        "应该只过滤出存在的1个函数"
    );
    assert_eq!(
        mixed_registry.get_supported_function_names()[0],
        "git-status",
        "应该是git_status"
    );

    // 测试总结
    println!("\n🎉 工具过滤功能测试完成总结:");
    println!("  ✅ 全局注册表初始化: 正常");
    println!("  ✅ 指定工具列表过滤: 正常");
    println!("  ✅ 单个工具过滤: 正常");
    println!("  ✅ 空工具列表处理: 正常");
    println!("  ✅ GxAIFun 工具过滤集成: 正常");
    println!("  ✅ with_tools 方法: 正常");
    println!("  ✅ 不存在工具处理: 正常");
    println!("  ✅ 混合工具列表处理: 正常");

    println!("\n🚀 所有工具过滤功能测试通过！");

    Ok(())
}
