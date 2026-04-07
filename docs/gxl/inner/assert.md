# gx.assert

## 作用

断言比较，默认比较“相等应成立”。

## 语法

```gxl
gx.assert(
  value: "<实际值>",
  expect: "<期望值>",
  err: "<失败提示>",
  result: "true|false"
);
```

参数：
- `value`：实际值
- `expect`：期望值
- `err`：失败时消息（可选）
- `result`：`"true"` 表示期望相等；`"false"` 表示期望不相等

## 示例

```gxl
gx.assert(value: "${ENV}", expect: "prod");
gx.assert(value: "${ENV}", expect: "prod", result: "false");
```
