# Galaxy Flow

Galaxy Flow 是一个面向流程编排的 DSL（GXL），提供两类 CLI：
- `gflow`：执行工作流（默认读取 `./_gal/work.gxl`）
- `gprj`：项目管理与管理流执行（默认读取 `./_gal/adm.gxl`）

## Current Status / 当前状态

- 运行时主链路可用：`parser -> model -> ability -> runner`
- 内置 `gx.*` 能力可用（见下文）
- `gprj self` 自更新可用（check/update/rollback）
- AI 能力当前为降级状态：
  - `ai_diagnose` 当前是 no-op（仅提示 `AI diagnose is currently disabled`）
  - `gx.ai_chat` 未作为当前内置 block 能力接入

## Core Capabilities / 核心能力

当前 parser 直接支持的内置能力：
- `gx.assert`
- `gx.cmd`
- `gx.echo`
- `gx.read_file` / `gx.read_cmd` / `gx.read_stdin`
- `gx.vars`（仅 env 内）
- `gx.tpl`
- `gx.ver`
- `gx.run`
- `gx.shell`
- `gx.tar` / `gx.untar`
- `gx.download` / `gx.upload`
- `gx.patch_file`
- 表达式函数：`defined(${VAR})`

详细说明见 `docs/gxl/inner/index.md`。

## 安装说明 / Installation

### 1. 一键安装（推荐）

```bash
curl -fsSL https://github.com/galaxy-sec/get/raw/main/install.sh | bash
```

默认安装到：`$HOME/bin`

可选参数：

```bash
# alpha channel
curl -fsSL https://github.com/galaxy-sec/get/raw/main/install.sh | bash -s -- --channel alpha

# custom install dir
curl -fsSL https://github.com/galaxy-sec/get/raw/main/install.sh | INSTALL_DIR=/usr/local/bin bash
```

安装后验证：

```bash
gprj --version
gflow --version
```

如果提示命令不存在，请把安装目录加入 `PATH`（例如 `$HOME/bin`）。

## Quick Start

### Build

```bash
cargo build --workspace
```

如果本机启用了 `sccache` 且报错，可临时关闭：

```bash
RUSTC_WRAPPER='' cargo build --workspace
```

### Initialize Project

```bash
gprj init env
gprj init prj --tpl simple
```

### Run Flows

```bash
# 查看工作流信息
gflow

# 运行工作流中的 conf flow
gflow conf

# 运行管理流中的 conf flow
gprj adm conf
```

## CLI Overview

### gflow

```bash
gflow [OPTIONS] [FLOWS]...
```

常用参数：`-e/--env`、`-c/--conf`、`-d/--debug`、`--cmd-arg`、`--dryrun`、`--ai`、`--mod_up`

### gprj

```bash
gprj <COMMAND>
```

子命令：
- `init`
- `update`
- `adm`
- `conf`
- `check`
- `self`

## Self Update (`gprj self`)

```bash
gprj self status
gprj self check --channel <stable|alpha|beta>
gprj self update --channel <stable|alpha|beta> [--to <version>] [--dry-run] [--force] --yes
gprj self rollback [--id <backup_id>]
```

说明：
- `rollback` 参数是 `--id`，不是 `--backup-id`
- 更新包下载到临时目录，完成后清理
- 成功更新后会替换当前安装目录中的 `gprj` 和 `gflow`
- 备份与状态目录：
  - `~/.galaxy/self_update/state.json`
  - `~/.galaxy/self_update/backups/<backup_id>/`

## `gx.patch_file` Notes

- `strict` 默认 `true`，语义是：marker 结构异常即失败
- 注释/反注释动作要求 `comment_prefix` 非空
- marker 支持 `@gxl:*`，并兼容 `#@gxl:*`、`//@gxl:*`（匹配 token 本体）

## Docs

- 本仓库使用指南：`docs/guidle/index.md`
- GXL 语法：`docs/gxl/syntax.md`
- 内置能力：`docs/gxl/inner/index.md`
- 结构文档（对齐代码）：`docs/structure/project-structure-actual.md`
- GitHub Pages: https://galaxy-sec.github.io/gxl-docs/
- DeepWiki: https://deepwiki.com/galaxy-sec/galaxy-flow

## Release

GitHub Releases:
- https://github.com/galaxy-sec/galaxy-flow/releases
