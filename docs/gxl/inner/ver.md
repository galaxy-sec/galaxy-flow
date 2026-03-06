# gx.ver

## 作用

读取并递增版本号（写回文件，并导出变量）。

## 语法

```gxl
gx.ver(
  file: "./version.txt",
  inc: "build|bugfix|feature|main|null"
);
```

版本格式：
- `major.minor.patch`
- `major.minor.patch.build`

说明：
- 默认导出变量为 `VERSION`。
- `export` 参数在当前解析层存在实现偏差，不建议使用。
