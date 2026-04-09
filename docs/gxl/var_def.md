# 变量定义

## 1. 赋值

```gxl
flow conf {
  ONE = "one";
  SYS_A = { MOD1: "A", MOD2: "B", MOD3: 1, MOD4: 2 };
  SYS_B = ["C", "D"];
  SYS_C = ${SYS_B[1]};
  SYS_D = ${SYS_A.MOD1};
}
```

## 2. 数据类型

支持：
- 字符串：`"text"` 或 `r#"raw text"#`
- 布尔：`true/false`
- 数字：整数、浮点
- 对象：`{ KEY: VALUE, ... }`
- 列表：`[VALUE, VALUE, ...]`
- 变量引用：`${VAR}`、`${OBJ.KEY}`、`${ARR[0]}`

## 3. 大小写规则

运行时变量键按不区分大小写处理（内部统一大写键）。

例如：
- `${SYS_A.MOD1}` 与 `${sys_a.mod1}` 读取同一个值。

## 4. 典型遍历

```gxl
mod envs {
  env default {
    DATA_LIST = ["JAVA", "RUST", "PYTHON"];
    DATA_OBJ = {
      JAVA: { NAME: "JAVA", SCORE: 80 },
      RUST: { NAME: "RUST", SCORE: 100 },
      PYTHON: { NAME: "PYTHON", SCORE: 200 }
    };
  }
}

mod main {
  flow list_do {
    for ${CUR} in ${ENV_DATA_LIST} {
      gx.echo(value: "CUR=${CUR}");
    }
  }

  flow obj_do {
    for ${CUR} in ${ENV_DATA_OBJ} {
      gx.echo(value: "${CUR.NAME}:${CUR.SCORE}");
    }
  }
}
```
