# Vars 示例

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
  flow array_do {
    for ${CUR} in ${ENV_DATA_LIST} {
      gx.echo(value: "CUR:${CUR}");
    }
  }

  flow obj_do {
    for ${CUR} in ${ENV_DATA_OBJ} {
      gx.echo(value: "CUR:${CUR.NAME}:${CUR.SCORE}");
    }
  }
}
```
