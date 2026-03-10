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
- `update`：更新项目模块
- `doc`：快速查看 GXL/CLI 文档主题
- `conf`：初始化本地配置
- `check`：检查当前运行环境
- `self`：自更新（status/check/update/rollback）

## 常用示例

```bash
# 初始化运行环境与项目
gx init env
gx init prj --tpl simple

# 运行工作流
gx run conf

# 运行管理流
gx adm conf

# 查看帮助主题
gx doc
gx doc gx.cmd
gx doc --markdown gx.cmd

# 更新模块
gx update mod
```

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
