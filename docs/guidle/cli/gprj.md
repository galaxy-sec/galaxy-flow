# gprj 命令使用文档

`gprj` 用于项目初始化、配置管理、模块更新、管理流执行和自更新。

## 基本用法

```bash
gprj <COMMAND>
```

## 命令总览

- `init`：初始化环境或项目
- `update`：更新项目模块
- `adm`：执行管理流（基于 `gflow` 参数模型）
- `conf`：初始化配置
- `check`：检查当前运行环境信息
- `self`：自更新（check/update/rollback）

## 1. init

### 1.1 初始化环境

```bash
gprj init env
```

### 1.2 本地模板初始化

```bash
gprj init prj-with-local
```

### 1.3 远程模板初始化

```bash
gprj init prj [OPTIONS]
```

参数：
- `--tpl <TPL>`：模板名，默认 `simple`
- `--branch <BRANCH>`：模板分支
- `--tag <TAG>`：模板 tag
- `--repo <REPO>`：模板仓库地址
- `-d, --debug <DEBUG>`：调试级别
- `--log <LOG>`：日志配置
- `-p, --cmd-print`：打印执行命令

## 2. update

### 2.1 更新模块

```bash
gprj update mod [OPTIONS]
```

参数：
- `-d, --debug <DEBUG>`：调试级别
- `--conf-work <FILE>`：默认 `./_gal/work.gxl`
- `--conf-adm <FILE>`：默认 `./_gal/adm.gxl`
- `--log <LOG>`：日志配置
- `-q, --quiet`：静默模式

## 3. adm

`gprj adm` 的参数与 `gflow` 一致，默认配置为 `./_gal/adm.gxl`。

示例：

```bash
# 运行 adm.gxl 中的 conf flow
gprj adm conf

# 指定环境
gprj adm -e dev conf
```

## 4. conf

```bash
gprj conf init
```

用于初始化项目配置文件。

## 5. check

```bash
gprj check
```

输出当前系统与运行环境信息。

## 6. self（自更新）

### 6.1 查看状态

```bash
gprj self status
```

### 6.2 检查更新

```bash
gprj self check --channel <stable|alpha|beta> [--json]
```

### 6.3 执行更新

```bash
gprj self update --channel <stable|alpha|beta> [--to <VERSION>] [--dry-run] [--force] --yes
```

参数：
- `--channel`：必填
- `--to`：期望目标版本（可选）
- `--yes`：确认执行更新
- `--dry-run`：仅检查不安装
- `--force`：即使版本未变也执行安装流程

### 6.4 回滚

```bash
gprj self rollback [--id <BACKUP_ID>]
```

- 不传 `--id`：回滚到最近备份
- 传 `--id`：回滚指定备份

## 示例流程

```bash
# 1) 初始化项目
gprj init env
gprj init prj --tpl simple

# 2) 更新模块
gprj update mod

# 3) 运行管理流
gprj adm conf

# 4) 自更新检查与安装
gprj self check --channel alpha
gprj self update --channel alpha --yes

# 5) 需要时回滚
gprj self rollback --id 20260306004548
```
