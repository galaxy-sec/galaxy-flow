# Galaxy Flow

Galaxy Flow 是基于 GXL 的开源自动化工作流引擎，提供自动化编排与执行底座，当前以 `gx` 作为统一 CLI 入口：
- `gx run`：执行工作流（默认读取 `./_gal/work.gxl`）
- `gx adm`：执行管理流（默认读取 `./_gal/adm.gxl`）
- `gx init/mod/doc/check/self`：项目与工具管理

在整体产品分工中：
- `galaxy-flow` 负责流程定义与执行
- `galaxy-ops` 负责运维能力的组织、配置与交付

## Current Status / 当前状态

- 运行时主链路可用：`parser -> model -> ability -> runner`
- 内置 `gx.*` 能力可用（见下文）
- `gx self` 自更新可用（check/update/rollback）
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
curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx
```

默认安装到：`$HOME/bin`

可选参数：

```bash
# alpha channel
curl -sSf https://get.warpparse.ai/inst-x.sh | bash -s -- gx alpha

# custom install dir
curl -sSf https://get.warpparse.ai/inst-x.sh | INSTALL_DIR=/usr/local/bin bash -s -- gx
```

安装后验证：

```bash
gx --version
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
# 初始化运行环境（首次使用）
gx init env

# 初始化项目（本地，不依赖远程模板）
gx init project

# 初始化项目（使用远程模板）
gx init project --tpl simple
gx init project --tpl simple --repo https://your-tpl-repo.git
```

其中 `gx init env` 会初始化 `~/.galaxy/` 下的用户级运行配置，包括 `conf.toml`。

`gx init project` 不带 `--tpl` 时，执行本地初始化，创建基本的 `./_gal/work.gxl` 和 `./_gal/adm.gxl` 文件。

### Run Flows

```bash
# 查看工作流信息
gx run

# 运行工作流中的 conf flow
gx run conf

# 运行管理流中的 conf flow
gx adm conf
```

## CLI Overview

### gx

```bash
gx <COMMAND>
```

常用子命令：
- `run`
- `adm`
- `init`
- `mod`
- `doc`
- `check`
- `self`

### gx run

```bash
gx run [OPTIONS] [FLOWS]...
```

常用参数：`-e/--env`、`-c/--conf`、`-d/--debug`、`--cmd-arg`、`--dryrun`、`--ai`

## Self Update (`gx self`)

```bash
gx self status
gx self check --channel <stable|alpha|beta>
gx self update --channel <stable|alpha|beta> [--to <version>] [--dry-run] [--force] --yes
gx self rollback [--id <backup_id>]
```

说明：
- `rollback` 参数是 `--id`，不是 `--backup-id`
- 更新包下载到临时目录，完成后清理
- 成功更新后会替换当前安装目录中的 `gx`
- 备份与状态目录：
  - `~/.galaxy/self_update/state.json`
  - `~/.galaxy/self_update/backups/<backup_id>/`
- Manifest 来源：`https://raw.githubusercontent.com/galaxio-labs/get/main/updates/gx/{channel}/manifest.json`

## `gx.patch_file` Notes

- `strict` 默认 `true`，语义是：marker 结构异常即失败
- 注释/反注释动作要求 `comment_prefix` 非空
- marker 支持 `@gxl:*`，并兼容 `#@gxl:*`、`//@gxl:*`（匹配 token 本体）

## Docs

- 本仓库使用指南：`docs/guidle/index.md`
- GXL 语法：`docs/gxl/syntax.md`
- 内置能力：`docs/gxl/inner/index.md`
- 结构文档（对齐代码）：`docs/structure/project-structure-actual.md`
- GitHub Pages: https://galaxio-labs.github.io/gxl-docs/
- DeepWiki: https://deepwiki.com/galaxio-labs/galaxy-flow

## Release

GitHub Releases:
- https://github.com/galaxio-labs/galaxy-flow/releases
