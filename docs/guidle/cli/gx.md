# gx 命令使用文档

`gx` 是 Galaxy Flow 的统一命令入口。

它将原来的工作流执行、管理流执行、项目初始化、模块更新、文档查看、自更新等能力收敛到一个命令下。

## 基本用法

```bash
gx <COMMAND>
```

## 命令总览

- `run`：执行工作流，默认配置 `./_gal/work.gxl`
- `adm`：执行管理流，默认配置 `./_gal/adm.gxl`
- `init`：初始化环境或项目
- `mod`：项目模块管理
- `doc`：快速查看 GXL/CLI 文档主题
- `check`：检查当前运行环境
- `self`：自更新（status/check/update/rollback）

## 常用示例

```bash
# 初始化运行环境（首次使用）
gx init env

# 初始化项目（本地，不依赖远程模板）
gx init project

# 初始化项目（使用远程模板）
gx init project --tpl simple
gx init project --tpl simple --repo https://your-tpl-repo.git
gx init project --tpl simple --branch dev

# 运行工作流
gx run conf

# 运行管理流
gx adm conf

# 查看帮助主题
gx doc
gx doc gx.cmd
gx doc --markdown gx.cmd

# 更新模块
gx mod update
```

## init project 详解

```bash
# 本地初始化（推荐，无需远程依赖）
gx init project

# 使用远程模板初始化
gx init project --tpl <template_name>
gx init project --tpl <template_name> --repo <git_repo_url>
gx init project --tpl <template_name> --branch <branch_name>
gx init project --tpl <template_name> --tag <tag_name>
```

说明：
- 不带 `--tpl` 参数时，执行本地初始化，创建基本的 `./_gal/work.gxl` 和 `./_gal/adm.gxl` 文件
- 带 `--tpl` 参数时，从指定的 git 仓库拉取模板（默认 `https://gal-tpl.git`）

## 兼容入口

- 如果做了链接，`grun ...` 等价于 `gx run ...`
- 如果做了链接，`gadm ...` 等价于 `gx adm ...`

## 目录约定

- `./_gal/work.gxl`：默认工作流入口，给 `gx run`
- `./_gal/adm.gxl`：默认管理流入口，给 `gx adm`
- `./_gal/mods/`：项目本地模块目录

## 文档阅读建议

- 想看 CLI 总览：`gx doc gx`
- 想看能力细节：`gx doc gx.cmd`、`gx doc gx.patch_file`
