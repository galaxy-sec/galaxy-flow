# Function 示例

```gxl
extern mod os { path = "../../_gal/mods"; }

mod sys {
  fn echo(name) {
    gx.echo(value: "echo:${name}");
  }

  fn echo_obj(obj) {
    gx.echo(value: "echo_obj:${obj}");
  }

  fn echo_list(list) {
    gx.echo(value: "echo_list:${list}");
  }
}

mod envs {
  env default {
    DATA = ["JAVA", "RUST", "PYTHON"];
    OBJ = { name: "test", value: "value" };
  }
}

mod main {
  flow conf {
    sys.echo(name: "test");
    sys.echo_obj(obj: "${OBJ}");
    sys.echo_list(list: "${DATA}");
  }
}
```
