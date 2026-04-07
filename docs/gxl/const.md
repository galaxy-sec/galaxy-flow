# GXL 内置常量

以下常量由运行时自动注入到变量空间。

- `GXL_PRJ_ROOT`
  - 从当前目录向上查找 `_gal/project.toml` 所在目录。
  - 找不到时值为 `UNDEFIN`。

- `GXL_GIT_BRANCH`
  - 以 `GXL_PRJ_ROOT`（或当前启动目录）为起点探测 git 分支名。
  - detached HEAD 或非 git 仓库时值为 `UNDEFIN`。

- `GXL_START_ROOT`
  - 启动 `gx` 时的工作目录。

- `GXL_CUR_DIR`
  - 当前执行目录；在 `gx.run` 切换目录时可能与 `GXL_START_ROOT` 不同。

- `GXL_CMD_ARG`
  - CLI 透传参数（来自 `--cmd-arg`）。

- `GXL_CMD_DRYRUN`
  - CLI dryrun 标记。

- `GXL_CMD_MODUP`
  - CLI 模块更新标记。

- `GXL_OS_SYS`
  - 系统标识字符串（由架构/系统/主版本组合）。
