# GXL AI-Native语法设计
# Version: 2.0 - AI-Agent Workflow Language
# 重新定位：GXL = 原生AI驱动的工作流语言

## 核心设计宣言
> GXL从现在开始是AI-first的语言。每一个任务都可以用AI增强，每一个工作流都能理解代码、意图和人类需求。

## AI原生语法规则

### 1. 大模型作为一等公民
```
ai <model> [options] {config} -> expression
```

### 2. 上下文感知分析指令
```
ai_context {
    files: [list]           # 文件范围
    diff: [boolean]        # 是否分析变更
    history: [int]         # 提交历史深度
    language: [string]     # 代码语言
}
```

### 3. 智能能力动词
```
- ai_analyze():    深度代码理解
- ai_suggest():    基于上下文建议  
- ai_check():      问题检测和审查
- ai_generate():   代码/文档创建
- ai_refactor():   重构建议
- ai_deploy():     智能部署决策
```

## AI-GXL完整语法规范

### 3.1 变更意图理解　　　　# Git Commit验证
```gxl
mod ai_git {

  flow smart_commit {
    # GXL原生读取git状态
    status = git.status(porcelain: true);
    
    if ai.has_changes("${status}") {
      changes = git.diff(cached: true, unified: 3);
      context = ai_context(
        files: "*.rs, *.py",
        pattern: "public_interface"
      );
      
      # GXL原生AI调用
      commit_msg = ai.gpt4(
        system: "你是一个代码变更理解专家，生成符合conventional commits的简洁消息",
        prompt: "分析以下代码变更并生成提交信息：\n${changes}",
        context: "${context}"
      );
      
      confirmed = ai.confirm(
        prompt: "${commit_msg}",
        timeout: 30,
        edit_mode: true
      );
      
      if confirmed.accept {
        git.add(all: true);
        git.commit(message: confirmed.final);
      }
+    }
+  }
}
```

### 3.2 代码质量AI检查
```gxl
mod ai_quality {

  flow daily_review {
    repo_context = ai_context(
      diff_range: "HEAD~3..HEAD",
      complexity_threshold: 15,
      security_focus: true
    );
    
    # AI native安全配置
    security_report = ai.claude3_5(
      purpose: "security_audit",
      scope: "${repo_context}",
      format: "markdown_issue"
    );
+    
+    # 直接AI交互，无需shell
+    if security_report.critical_count > 0 {
+      ai.create_issue(
+        title: "Security Alert: ${security_report.headline}",
+        body: security_report.details,
+        assignees: ["security-team"]
+      );
+    }
  }
}
```

### 3.3 AI文档自动生成
```gxl
mod ai_docs {

  flow changelog_next {
    # GXL直接理解历史
    recent_changes = git.log(limit: 5, format: "full");
    
    auto_changelog = ai.gpt4o(
      task: "generate_changelog",
      changes: "${recent_changes}",
      style: "conventional_commits",
      audience: "developers"
    );
    
    # 智能格式化
    ai.write_file(
      path: "CHANGELOG.md.template",
      content: auto_changelog.formatted,
      insert_at: "## [Unreleased]"
    );
  }
  }
 }
```

## 4. AI原生能力矩阵

| 能力层级 | GXL语法 | 作用范围 | 使用场景 |
+|---|---|---|---|
+| 文件理解 | `ai.analyze('file.rs')` | 源码深度分析 | 复杂度、安全、风格检查 |
+| 变更理解 | `ai.diff_explain('HEAD~1..HEAD')` | 版本差异意图 | Git commit、PR描述 |
+| 项目全景 | `ai.understand_project()` | 整体架构认知 | 重构建议、文档生成 |
+| 意图预测 | `ai.predict_impact('feature_branch')` | 变更影响分析 | 合并风险、回归测试 |
+| 协作增强 | `ai.team_context('pr_review')` | 团队协作优化 | 代码review、知识传递 |

## 5. 原生配置系统
```gxl
# 全局GXL AI配置
env ai_config {
  default_model = "gpt-4o"
  backup_models = ["claude-3-5-sonnet", "ollama/deepseek-coder"]  
  timeout = 30
  temperature = 0.7
  interactive = true
  filter_sensitive = true
  
  # Token管理策略
  token_limits = {
    commit: 150,
    review: 2000,
    analysis: 4000
  }

  ## 6. AI语法规范演示

  ### 6.1 工作流定义范式
  ```gxl
  # AI驱动的GxL工作流新模式

  mod ai_workspace {
    env global_settings {
      openai_key = "${OPENAI_API_KEY}"
  +    claude_key = "${CLAUDE_API_KEY}" 
  +    default_model = "gpt-4o"  # 推荐模型
  +    timeout = 30             # 请求超时（秒）
  +  }
  +
  +  # AI原生场景：智能Git提交（无需任何shell脚本）
  +  flow ai_smart_commit : global_settings {
  +    # GXL原生收集变更信息
  +    unstaged_files = git.list(modified: true)
  +    staged_files = git.list(staged: true)
  +    
  +    # 智能区分变更类型
  +    source_changes = file.select("${unstaged_files} ${staged_files}" 
  +      patterns: ["*.rs", "*.py", "*.js"], last_only: true)
  +    
  +    if [ -z "${source_changes}" ] {
  +      mx.echo "🎉 没有代码变更需要提交"
  +      exit 0
  +    }
  +    
  +    # AI直接理解变更意图（零中间层）
  +    file_diffs = git.diff(files: "${source_changes}", context: 3)
  +    
  +    commit_analysis = ai.gpt4o(
  +      instructions: "作为一名资深工程师，分析这些代码变更的关键点，生成一个符合Conventional Commits标准、最多50字的英文提交消息，控制在单行内。重点突出功能改进或问题解决的核心。",
  +      content: "${file_diffs}",
  +      mode: "single_line",
  +      strict: true
  +    )
  +    
  +    # 交互确认与编辑支持
  +    final_msg = gx.prompt(
  +      message: "AI建议的提交信息：\n${commit_analysis}\n\n接受(y)或编辑(e)？",
  +      accept_key: "y",
  +      edit_key: "e",
  +      default: "${commit_analysis}"
  +    )
  +    
  +    # 直接git操作完成智能提交
  +    git.add(include: "${source_changes}")
  +    git.commit(message: "${final_msg}")
  +    
  +    mx.echo "✅ AI智能提交完成：${final_msg}"
  +  }
  +  
  +  # 横向扩展：代码质量
  +  flow ai_code_review : global_settings {
  +    changed_files = git.changes(since_branch: "main")
  +    
  +    quality_report = ai.claude35(
  +      task: "comprehensive_code_review",
  +      target_files: "${changed_files}",
  +      focus_areas: ["security", "performance", "maintainability"],
  +      format: "markdown_report"
  +    )
  +    
  +    gx.display(content: "${quality_report}", format: "markdown")
  +    
  +    if [ "${quality_report.risk_level}" = "high" ] {
  +      gx.confirm("发现实施性问题，是否继续？(y/n)")
  +    }
  +  }
  +}
  +```
  +
  +## 7. 实现极简路径
  +
  +AI原生实现仅需：
  +```
  +☑ 系统基础 (已存在): 系统Git、已安装GxL、网络连接
  +☑ 环境变量: OPENAI_API_KEY / CLAUDE_API_KEY
  +☑ 配置: ~/.gflow/ai-config.yaml (无需编译，纯YAML配置)
  +☑ 启动: gx run ai_smart_commit
  +```
  +
  +## 8. GXL核心价值声明
  +> GXL现在是一种**AI-first工作流语言**，能够直接理解人类意图、代码语义和协作需求。Git Commit验证只是众多AI原生场景中的第0个验证用例。
+  
+  # Model routing
+  routing_rules = {
+    simple: "gpt-4o-mini",
+    complex: "claude-3-5-sonnet",
+    free: "ollama/codellama"
+  }
}
