# gx.patch_file

## 作用

基于 marker 对文件做受控修改，支持：
- `set`
- `comment_line`
- `uncomment_line`
- `comment_block`
- `uncomment_block`

## 语法

```gxl
gx.patch_file(
  file: "./Cargo.toml",
  action: "set",
  marker: "version",
  value: "0.12.3",
  strict: "true",
  dry_run: "false",
  backup: "false",
  comment_prefix: "#"
);
```

参数：
- `file`：目标文件（必填）
- `action`：操作类型（必填）
- `marker`（或 `id`）：marker id（必填）
- `value`：仅 `set` 需要
- `strict`：默认 `true`
- `dry_run`：默认 `false`
- `backup`：默认 `false`
- `comment_prefix`（或 `comment`）：默认 `#`

## marker 约定

- set：`@gxl:set(<id>)`
- line：`@gxl:line(<id>)`
- block start：`@gxl:block(<id>)`
- block end：`@gxl:end(<id>)`

兼容写法：
- `# @gxl:...`
- `#@gxl:...`
- `// @gxl:...`
- `//@gxl:...`

## strict 语义

`strict=true` 时，任何 marker 结构异常都失败，例如：
- marker 命中数不是 1
- block 嵌套
- 只有 `end` 没有 `block`
- 缺少 `end`

## comment_prefix 约束

- `comment_prefix` 不能为空字符串。

## 示例

### 1) set

```toml
version = "0.12.2"   # @gxl:set(version)
```

```gxl
gx.patch_file(
  file: "./Cargo.toml",
  action: "set",
  marker: "version",
  value: "\"0.12.3\""
);
```

### 2) comment/uncomment block

```toml
# @gxl:block(res_depend_test)
[features]
res_depend_test = []
# @gxl:end(res_depend_test)
```

```gxl
gx.patch_file(file: "./Cargo.toml", action: "comment_block", marker: "res_depend_test");
gx.patch_file(file: "./Cargo.toml", action: "uncomment_block", marker: "res_depend_test");
```

### 3) line toggle

```toml
res_depend_test = []   # @gxl:line(res_depend_test)
```

```gxl
gx.patch_file(file: "./Cargo.toml", action: "comment_line", marker: "res_depend_test");
gx.patch_file(file: "./Cargo.toml", action: "uncomment_line", marker: "res_depend_test");
```
