# GF EBNF

以下是 GF EBNF 语法的详细规则，包含注释解释每个规则的作用。

```EBNF
-- 模块定义，以 "mod" 关键字开始，后跟模块名称和可选的命名空间，包含属性和环境/流程定义
MOD ::= "mod"  NAME [":" {(NAME ",")} ] "{"  {PROPERTY}*   {(ENV|FLOW)}*  "}"
-- 环境定义，以 "env" 关键字开始，后跟环境名称和可选的命名空间，包含属性和动作调用
ENV ::= "env"  NAME [":" {(NAME ",")} ]"{"  {PROPERTY}*   {ACTCALL}* "}"
-- 流程定义，以 "flow" 关键字开始，后跟流程名称和可选的命名空间，包含属性和动作调用
FLOW ::= "flow"  NAME [":" {(NAME ",")}+ [":" {(NAME ",")}+]] "{"  {PROPERTY}*  {ACTCALL}* "}"
-- 属性定义，由名称、等号和字符串值组成，以分号结尾
PROPERTY ::= NAME "="  STRING ";"
-- 动作调用，由动作名称和属性列表组成，可选以分号结尾
ACTCALL  ::=  ACT_NAME "{" {PROPERTY}+ "}" [";"]
-- 动作名称，可以包含命名空间
ACT_NAME ::= [NAME "."] NAME
-- 字符串定义，由双引号包围
STRING ::= "\"" {[^"]}* "\""
-- 名称定义，由字母和数字组成
NAME ::=  {[a-zA-Z0-9]}+
```


```
env dev {
	root = "${HOME}/my_project";
	gx.read {
		name = "MY_PATH" ;
		cmd  = "pwd" ;
	};
}
```

```
mod my_module {
    -- 模块属性
    author = "John Doe";
    version = "1.0";

    -- 环境定义
    env test {
        root = "${HOME}/test_project";
        gx.read {
            name = "TEST_PATH";
            cmd = "ls";
        };
    }

    -- 流程定义
    flow my_flow {
        step1 = "execute_task";
        task1.run {
            param1 = "value1";
            param2 = "value2";
        };
    }
}
```