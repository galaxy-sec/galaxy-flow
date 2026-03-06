# gx.cmd

## 作用

执行一条命令字符串。

## 语法

```gxl
gx.cmd(
  cmd: "<command>",
  err: "<err var>",
  suc: "<success marker>",
  sudo: "true|false",
  log: "1|2|3",
  silence: "true|false",
  quiet: "true|false"
);
```

也支持匿名首参数：

```gxl
gx.cmd("echo hello");
```

## 代码块命令

````gxl
```cmd
cp a b
ls -al
```
````

## 示例

```gxl
gx.cmd(cmd: "git branch --show-current");
gx.cmd("echo ${HOME}");
```
