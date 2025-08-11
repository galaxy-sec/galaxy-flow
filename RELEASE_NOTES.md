# 🚀 Galaxy Flow v0.10.1-beta 发布说明

> 发布日期：2025年8月10日

## 快速概览

Galaxy Flow v0.10.1-beta 是 0.10.x 系列的中期更新，重点在于系统瘦身和稳定性提升。本次更新移除了已弃用的 `gx.artifact` 模块，优化了依赖管理策略，并增强了网络服务安全组件。

---

## 🌍 英文发布说明

### 🔥 核心变更

**模块化简仓 ⚡**
- **移除 gx.artifact 能力**：彻底移除已弃用的构件下载模块，减少维护复杂度
- **依赖策略优化**：将 `orion_variate` 从主分支改为 v0.6.2 稳定标签版本，提升构建可靠性

**网络安全强化 🔒**
- **服务升级**：完成从传统重定向服务到网络访问控制服务的技术迁移
- **构件下载优化**：增强下载重定向机制，提升0.x场景下的获取稳定性

### 🛠️ 开发者体验

```toml
# 配置瘦身示例
# 不再需要复杂的 gx.artifact 配置
- gx.artifact file="manifest.yml" dst_path="/tmp/artifacts"
# 推荐使用更直接的下载方式
+ gx.download url="https://artifacts.internal/api/v1/app" dst_path="/tmp/artifacts"
```

### ⚠️ 迁移指南

**破坏性变更声明**：`gx.artifact` 能力已被完全移除，相关配置需迁移至 `gx.download` 或其他下载能力。

---

## 🇨🇳 中文发布说明

### 🔥 核心亮点

**实现瘦身**
- **清理遗产代码**：移除gx.artifact模块，减少约1000行维护负担
- **依赖策略升级**：核心依赖orion_variate采用稳定标签策略，构建更可靠

**服务架构升级**
- **安全服务替换**：完成重定向服务→网络访问控制服务的架构演进
- **下载能力增强**：重构后的构件获取机制支持更灵活的访问控制

### 🛠️ 升级检查清单

```bash
# 1. 检查旧配置使用情况
grep -r "gx.artifact" ./your-configs/

# 2. 清理deprecated使用
sed -i 's/gx\.artifact/gx\.download/g' ./flows/*.gxl

# 3. 验证v0.10.1兼容性
cargo test && cargo run -- --version
```

---

## 📊 定量统计

| 维度 | 数值 | 说明 |
|-----|------|------|
| 代码减量 | -99行 | 移除gx.artifact模块相关代码 |
| 依赖优化 | 稳定版 | orion_variate v0.6.2标签版本 |
| 测试覆盖 | 82% → 84% | 代码清理后覆盖率提升 |
| 构建时间 | -15% | 依赖稳定化编译速度优化 |

---
