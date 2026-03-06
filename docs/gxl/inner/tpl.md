# gx.tpl

## 作用

模板渲染（当前仅 `handlebars` 引擎）。

## 语法

```gxl
gx.tpl(
  tpl: "<模板文件或模板目录>",
  dst: "<目标文件或目标目录>",
  data: "<json string>",
  file: "<json file>",
  engine: "handlebars"
);
```

参数：
- `tpl`：模板路径
- `dst`：输出路径
- `data`：内联 JSON 字符串（可选）
- `file`：JSON 数据文件（可选）
- `engine`：`handlebars|helm`（当前执行层仅支持 `handlebars`）

## 示例

```gxl
gx.tpl(
  tpl: "./conf/tpls",
  dst: "./conf/used",
  file: "./conf/value.json"
);
```
