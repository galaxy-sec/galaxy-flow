# gx.run

## 作用

在子目录中执行另一个 GXL 配置。

## 语法

```gxl
gx.run(
  local: "<run dir>",
  conf: "<gxl file>",
  env: "<env name>",
  flow: "a,b,c",
  isolate: "true|false"
);
```

参数：
- `local`：运行目录
- `conf`：目标配置文件（默认 `./_gal/work.gxl`）
- `env`：目标环境
- `isolate`：是否隔离变量空间
- `flow`：解析层可接受；当前执行层未实际覆盖 flow 列表（保留参数）
