# defined(...)

## 作用

条件表达式函数，判断变量是否已定义。

## 语法

```gxl
if defined(${HOME}) {
  gx.echo(value: "has home");
}

if !defined(${NO_SUCH_VAR}) {
  gx.echo(value: "missing");
}
```

说明：
- 这是表达式函数，不是 `gx.defined` 命令。
