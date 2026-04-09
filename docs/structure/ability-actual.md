# ability 模块实际结构

## 模块定位

`src/ability` 提供 GXL 执行时可调用的能力实现（命令、模板、读取、下载、补丁等）。

## 实际文件结构

```text
src/ability/
├── mod.rs
├── archive.rs
├── assert.rs
├── cmd.rs
├── delegate.rs
├── echo.rs
├── gxl.rs
├── load.rs
├── shell.rs
├── tpl.rs
├── version.rs
├── ai/
│   ├── mod.rs
│   └── tool.rs
├── patch/
│   ├── mod.rs
│   ├── controller.rs
│   ├── model.rs
│   └── view.rs
└── read/
    ├── mod.rs
    ├── cmd.rs
    ├── file.rs
    ├── integra.rs
    └── stdin.rs
```

## 对外导出（`src/ability/mod.rs`）

- 模块：`ai, archive, assert, cmd, delegate, echo, gxl, load, patch, prelude, read, tpl, shell, version`
- 常用类型重导出：`GxAssert, GxCmd, GxEcho, GxRead, GxTpl, GxlVersion, GxRun, GxDownLoad, GxUpLoad`

## 说明

- `ability::ai` 当前仅导出 `tool` 子模块；目录中历史文件（如 `ai_call.rs`）未在 `ai/mod.rs` 导出。
- `ability::patch` 为 `gx.patch_file` 等补丁能力的 MVC 组织。
