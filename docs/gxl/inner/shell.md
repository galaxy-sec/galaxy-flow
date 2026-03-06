# gx.shell

## 作用

执行 shell 命令/脚本，可加载参数文件，可回填输出变量。

## 语法

```gxl
gx.shell(
  shell: "<command or script>",
  arg_file: "<json|yml|yaml|toml|ini>",
  out_var: "<var name>",
  err: "<err var>",
  log: "1|2|3",
  sudo: "true|false",
  silence: "true|false"
);
```

也支持匿名首参数：

```gxl
gx.shell("./demo.sh");
```
