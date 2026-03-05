# 自动升级能力设计（基于当前 galaxy-flow 工程）

- 文档状态：Draft（修正版）
- 最后更新：2026-03-05

## 1. 现状与修正说明

这份文档原稿确实包含外部项目遗留内容，和当前仓库不一致，主要问题：

1. 出现了 `wproj` / `wparse` 等当前仓库不存在的二进制名称。
2. 验收标准写了“4 个二进制版本一致”，但当前仓库实际只有 `gprj`、`gflow` 两个 bin（见 `Cargo.toml`）。
3. CLI 路径引用了不存在的目录（如 `wproj/args.rs`）。
4. channel 策略描述与当前更新目录策略不一致（当前实现为 `stable/alpha/beta` 三通道）。

本修正版严格基于当前工程结构：

- CLI：`app/gprj/args.rs`, `app/gprj/main.rs`
- 运行器：`app/gflow/main.rs`
- 下载能力：`src/ability/load.rs`（可复用网络下载思路）
- 配置根目录：`~/.galaxy`（现有 `conf.toml` 已使用）
- 发布流水线：`.github/workflows/release.yml`

## 2. 目标与非目标

### 2.1 目标

1. 提供统一入口：`gprj self ...`，用于检查/升级/回滚。
2. 升级覆盖 `gprj` + `gflow` 两个可执行文件。
3. 升级过程可回滚、可审计，默认安全（不强制自动应用）。
4. 支持自动检查（auto-check），自动应用作为可选策略。

### 2.2 非目标（MVP）

1. 不做增量补丁（仅全量压缩包）。
2. 不做 GUI 提示。
3. 不在首版实现复杂多源镜像与断点续传。

## 3. CLI 设计（挂在 gprj）

在 `app/gprj/args.rs` 新增：

```bash
gprj self status
gprj self check [--channel stable|alpha|beta] [--json]
gprj self update [--channel <c>] [--to <version>] [--yes] [--dry-run] [--force]
gprj self rollback [--id <backup_id>]
gprj self auto enable|disable|set --interval <hours> --mode check|apply
```

说明：

1. `status`：本地版本、安装路径、策略、最近一次检查结果。
2. `check`：远端检查，不落盘。
3. `update`：下载、校验、替换、健康检查；失败自动回滚。
4. `rollback`：回滚到最近一次或指定备份。
5. `auto`：更新自动策略（默认仅 check）。

## 4. 配置与状态文件

沿用当前项目配置根目录 `~/.galaxy`，新增：

```text
~/.galaxy/self_update/
  policy.toml
  state.json
  lock
  backups/
```

`policy.toml` 示例：

```toml
enabled = true
mode = "check"          # check | apply
channel = "stable"      # stable | alpha | beta
interval_hours = 24
```

## 5. 发布通道与版本源（对齐当前 release.yml）

当前 `.github/workflows/release.yml` 的事实：

1. 触发条件：`push tags: v*.*.*`
2. 构建产物：`gprj` + `gflow` 打进 tar.gz
3. 预发布判定：tag 含 `-pre` -> `prerelease=true`

因此自动升级通道在当前工程定义为：

1. `stable`：稳定发布
2. `alpha`：alpha 预发布
3. `beta`：beta 预发布

## 6. 远端 Manifest 规范（新增）

建议在 release 产物旁新增每通道清单（由 CI 生成）：

```text
updates/
  stable/manifest.json
  alpha/manifest.json
  beta/manifest.json
```

`manifest.json`（MVP 最小字段）：

```json
{
  "version": "0.12.1",
  "channel": "stable",
  "published_at": "2026-03-05T12:00:00Z",
  "git_ref": "v0.12.1",
  "git_commit": "abcdef...",
  "assets": {
    "x86_64-unknown-linux-gnu": {
      "url": "https://.../galaxy-flow-v0.12.1-x86_64-unknown-linux-gnu.tar.gz",
      "sha256": "..."
    },
    "aarch64-apple-darwin": {
      "url": "https://.../galaxy-flow-v0.12.1-aarch64-apple-darwin.tar.gz",
      "sha256": "..."
    }
  }
}
```

一致性校验：

1. 拉取路径 channel == manifest.channel
2. manifest.version 与 asset 文件名版本一致
3. 目标平台 target 必须存在
4. 下载文件 sha256 必须匹配

## 7. 升级流程（MVP）

1. 读取当前版本：`env!("CARGO_PKG_VERSION")`
2. 读取策略（policy）并确定 channel
3. 拉取 manifest，比较 semver
4. 下载目标 tar.gz 到临时目录
5. 校验 sha256
6. 解压并验证包含 `gprj`、`gflow`
7. 获取 lock（防并发）
8. 备份当前 bin 到 `backups/<timestamp>/`
9. 原子替换（同目录 rename）
10. 健康检查：`gprj --version` / `gflow --version`
11. 成功写 `state.json`；失败自动回滚并写失败原因

## 8. 安全策略

MVP：

1. 强制 sha256 校验
2. 仅允许白名单域名（GitHub Releases/指定官方域）
3. 并发锁 + 回滚闭环

V2：

1. 增加 manifest 签名（ed25519）
2. 支持 key rotation

## 9. 与安装来源兼容

检测为包管理器安装（brew/apt/yum）时：

1. `check` 正常可用
2. `update` 默认提示用包管理器升级
3. `--force` 才允许覆盖（需确认）

## 10. 自动检查触发点

建议仅在 `gprj` 启动流程触发（轻量、可跳过）：

1. 放在 `app/gprj/main.rs` 的命令分发前后，异步执行或快速执行。
2. 默认只做 check，不阻塞主命令。
3. `gflow` 首版不做在线升级触发，避免运行面风险。

## 11. 代码落地位置

### 11.1 CLI 层

1. `app/gprj/args.rs`：新增 `Self` 子命令枚举
2. `app/gprj/main.rs`：新增 `GxAdmCmd::Self(...)` 分发

### 11.2 核心模块

新增：

```text
src/self_update/
  mod.rs
  model.rs      # policy/state/manifest 结构
  storage.rs    # ~/.galaxy/self_update 读写
  client.rs     # manifest 拉取 + 下载
  installer.rs  # 解压/备份/替换/回滚
  service.rs    # check/update/rollback 主流程
```

并在 `src/lib.rs` 暴露 `pub mod self_update;`。

### 11.3 可复用组件

1. HTTP 下载能力可参考 `src/ability/load.rs` 的 accessor 方案。
2. 统一错误风格复用 `src/err.rs` / `ExecReason` 模型。

## 12. CI/CD 最小改造

在现有 `.github/workflows/release.yml` 上新增步骤：

1. 计算每个 target 产物 sha256
2. 生成 `manifest.json`（stable/alpha/beta）
3. 上传 manifest 到 release 资产（或独立 updates 路径）

后续再加：

1. manifest 签名
2. 客户端签名验签

## 13. 验收标准（按当前项目修正）

1. 升级成功后 `gprj`、`gflow` 两个二进制版本一致且可执行。
2. 人为注入校验失败/替换失败时可自动回滚。
3. 并发触发更新不会破坏安装（lock 生效）。
4. 默认不误覆盖包管理器安装路径。
5. `stable`、`alpha`、`beta` 的检查/升级命中正确清单。

## 14. 分阶段实施建议

### Phase 1（MVP，可交付）

1. `gprj self status/check/update/rollback`
2. `policy.toml` + `state.json`
3. sha256 校验 + 备份回滚
4. CI 产出 manifest

### Phase 2（增强）

1. `gprj self auto ...` 完整策略
2. 启动时 auto-check
3. manifest 签名与验签

### Phase 3（高级）

1. 镜像源与离线包
2. 更细粒度指标与告警

---

本修正版可直接作为当前仓库的实现蓝图；下一步可按 Phase 1 开发并同步补充集成测试。
