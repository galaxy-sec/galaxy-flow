# Template 示例

```gxl
extern mod os { path = "../../_gal/mods"; }

mod base_env {
  env _common {
    gx.vars {
      DOMAIN = "domain";
      SOCK_FILE = "socket";
      GXL_PRJ_ROOT = "./";
    };
  }

  env cli : _common {
    ROOT = "./";
  }
}

mod envs : base_env {
  env default : cli;
}

mod main {
  conf = "${ENV_ROOT}/conf";

  flow conf {
    os.path(dst: "${MAIN_CONF}/used", keep: "true");

    gx.tpl(
      tpl: "${MAIN_CONF}/tpls",
      dst: "${MAIN_CONF}/used",
      file: "${MAIN_CONF}/value.json"
    );

    gx.cmd(cmd: "cp ./conf/value.json ./conf/used/back.json");
  }
}
```
