# gx.vars

## 作用

批量定义变量。

## 语法

```gxl
env default {
  gx.vars {
    APP = "galaxy";
    STAGE = "dev";
  };
}
```

说明：
- 当前语法是 `=` 赋值，支持 `;`/`,` 分隔。
- `gx.vars` 只在 `env` 中作为 env item 使用。
