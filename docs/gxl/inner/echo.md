# gx.echo

## 作用

输出一段文本到 stdout。

## 语法

```gxl
gx.echo(value: "<text>");
```

也支持匿名首参数：

```gxl
gx.echo("hello");
```

说明：
- 当前实现仅支持输出文本，不支持旧文档中的 `file/export/inc` 参数。
