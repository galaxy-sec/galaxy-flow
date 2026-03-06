# gflow 命令使用文档

`gflow` 用于执行工作流配置（默认 `./_gal/work.gxl`）。

## 基本用法

```bash
gflow [OPTIONS] [FLOWS]...
```

## 参数说明

- `FLOWS...`：要执行的 flow 名称（位置参数）
- `-e, --env <ENV>`：环境名，默认 `default`
- `-d, --debug <DEBUG>`：调试级别，默认 `0`
- `-c, --conf <CONF>`：指定 GXL 配置文件路径
- `--log <LOG>`：日志配置，如 `cmd=debug,parse=info`
- `-q, --quiet`：静默模式
- `--cmd-arg <ARG>`：传递给流程的命令参数
- `--dryrun`：试运行模式，不真正执行
- `--ai`：出错时启用 AI 诊断
- `--mod_up`：更新远程模块

## 默认行为

- 未指定 `--conf` 时，默认使用 `./_gal/work.gxl`
- 未指定任何 `FLOWS` 时，会展示可执行信息（不执行具体 flow）

## 示例

```bash
# 使用默认配置执行 conf flow
gflow conf

# 指定环境与配置文件
gflow -e dev -c ./_gal/work.gxl conf

# 试运行
gflow --dryrun conf

# 传递命令参数
gflow --cmd-arg "--name demo" conf

# 开启 AI 诊断
gflow --ai conf
```

## 常见问题

### 配置文件找不到

确认 `./_gal/work.gxl` 存在，或显式传入 `-c`。

### 变量不生效

优先检查：
- 环境块是否匹配 `--env`
- 变量引用格式是否正确（如 `${VAR}`）
- 是否被后续步骤覆盖
