use serde::{Deserialize, Serialize};
use strum_macros::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum AiCapability {
    /// 深度代码理解 - 分析代码复杂度、结构、模式等
    Analyze,

    /// 基于上下文的智能建议
    Suggest,

    /// 问题检测和审查 - 安全、性能、可维护性
    Check,

    /// 代码/文档/配置生成
    Generate,

    /// 重构建议 - 改进代码结构和质量
    Refactor,

    /// 智能部署决策 - 部署策略和风险评估
    Deploy,

    /// Git提交信息生成 - 基于代码变更理解
    Commit,

    /// 代码审查和PR建议
    Review,

    /// 项目整体架构理解
    Understand,

    /// 变更影响分析
    Predict,

    /// 团队协作增强
    Collaborate,
}

impl AiCapability {
    /// 检查该能力是否需要完整代码上下文
    pub fn needs_full_context(&self) -> bool {
+        matches!(
+            self,
+            AiCapability::Review | AiCapability::Understand | AiCapability::Predict
+        )
+    }
+
+    /// 检查该能力是否对token敏感
+    pub fn is_token_sensitive(&self) -> bool {
+        matches!(
+            self,
+            AiCapability::Generate | AiCapability::Refactor | AiCapability::Understand
+        )
+    }
+
+    /// 获取该能力的推荐模型
+    pub fn recommended_model(&self) -> &'static str {
+        match self {
+            AiCapability::Analyze => "gpt-4o-mini",
+            AiCapability::Suggest => "gpt-4o-mini",
+            AiCapability::Check => "gpt-4o",
+            AiCapability::Generate => "gpt-4o",
+            AiCapability::Refactor => "claude-3-5-sonnet",
+            AiCapability::Deploy => "gpt-4o-mini",
+            AiCapability::Commit => "gpt-4o-mini",
+            AiCapability::Review => "claude-3-5-sonnet",
+            AiCapability::Understand => "claude-3-5-sonnet",
+            AiCapability::Predict => "gpt-4o",
+            AiCapability::Collaborate => "gpt-4o-mini",
+        }
+    }
+}
```

## 7. 创建AI客户端和管理器
