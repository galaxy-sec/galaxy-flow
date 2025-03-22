# Galaxy Flow 内置能力

### gx.cmd

执行shell 命令

```bash
gx.cmd { cmd = "${PRJ_ROOT}/do.sh";  err = "you err"; log = "1"; } ;
```



### gx.assert

断言

```bash
gx.assert { value = "hello" ; expect = "hello" ; err = "errinfo";}
```



### gx.echo

```bash
gx.echo { value = "${PRJ_ROOT}/test/main.py" ; }
```



### gx.read

读取

```bash
gx.read{ name = "RG"; cmd  = "echo galaxy-1.0"; err = "you err"; } ;
gx.read { ini = "vars.ini"; }
gx.read { stdin = "please input you name"; name  = "name";}
```



### gx.vars

```bash
 gx.vars {
    x = "${PRJ_ROOT}/test/main.py" ;
    y = "${PRJ_ROOT}/test/main.py" ;
} ;"
```



### gx.tpl

```bash
gx.tpl {
     tpl = "${PRJ_ROOT}/conf_tpl.toml" ;
     dst = "${PRJ_ROOT}/conf.toml";
     data = ^"{"branchs": ["develop","issue/11"]} "^;
}
```



### gx.ver

```bash
 gx.ver { file = "./version.txt" ;  inc = "bugfix" ; } 
```
##### inc : 
* bugfix 
* build
* feature
* main

##### export

version : 三位或位的版本号