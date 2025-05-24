
# GXL 文件语法

## GXL 语法定义
GXL（Galaxy Flow Language）是一种为 DevSecOps 自动化工作流设计的领域特定语言。根据代码库中的信息，GXL 的语法定义主要在 galaxy-sec/galaxy-flow 仓库的解析器模块中实现。

### GXL 语法结构
GXL 语言遵循模块化、层次化的结构，主要由以下核心组件组成：

* 模块（Modules）：使用 mod 关键字定义，是 GXL 的顶层组织单元
* 环境（Environments）：使用 env 关键字定义，用于配置不同的执行上下文
* 流程（Flows）：使用 flow 关键字定义，表示工作流程序
* 活动（Activities）：可重用的操作单元，在流程中调用


## EBNF 语法
```EBNF

; GXL 文件由一系列模块定义组成
GXL-File = *Module


(* GXL 文件由一系列模块定义组成 *)
GXL-File = {Module};

(* 模块定义 *)
Module = "mod", whitespace, ModuleName, whitespace, "{", whitespace, ModuleContent, whitespace, "}", whitespace, ";";
ModuleName = Identifier;
ModuleContent = {Property | Environment | Flow};

(* 属性定义 (键值对) *)
Property = PropertyName, whitespace, "=", whitespace, PropertyValue, whitespace, ";";
PropertyName = Identifier;
PropertyValue = String;

(* 环境定义 *)
Environment = "env", whitespace, EnvName, [whitespace, ":", whitespace, EnvRefList], whitespace, "{", whitespace, EnvContent, whitespace, "}";
EnvName = Identifier;
EnvContent = {Property};
EnvRefList = EnvRef, {",", whitespace, EnvRef};
EnvRef = Identifier;

(* 流程定义 - 两种形式：直接定义或引用其他流程 *)
Flow = DirectFlow | ReferenceFlow;

(* 直接定义流程 *)
DirectFlow = "flow", whitespace, FlowName, [whitespace, ":", whitespace, FlowRefList [ whitespace , ":" whitespace, FlowRefList ] ], whitespace, "{", whitespace, FlowContent, whitespace, "}", whitespace, ";";
FlowName = Identifier;
FlowContent = {Command};

(* 引用其他流程 *)
ReferenceFlow = "flow", whitespace, FlowName, whitespace, ":", whitespace, FlowRefList, whitespace, ";";
FlowRefList = FlowRef, {",", whitespace, FlowRef};
FlowRef = Identifier;


(* 命令定义 *)
Command = (BuiltinCommand | ActivityCall), whitespace, ";";

(* 内置命令 *)
BuiltinCommand = "gx.", CommandName, whitespace, "{", whitespace, CommandProps, whitespace, "}";
CommandName = "echo" | "vars" | "cmd" | "read" | "tpl" | "assert" | "ver";
CommandProps = {PropertyAssignment};
PropertyAssignment = PropertyName, whitespace, "=", whitespace, PropertyValue, whitespace,  ",";

(* 活动调用 *)
ActivityCall = ActivityName, whitespace, "{", whitespace, CommandProps, whitespace, "}";
ActivityName = Identifier, {".", Identifier};

(* 标识符 *)
Identifier = Alpha, {Alpha | Digit | "_"};
Alpha = "A" | "B" | ... | "Z" | "a" | "b" | ... | "z";
Digit = "0" | "1" | ... | "9";

(* 字符串 *)
String = '"', {StringChar}, '"';
StringChar = UnescapedChar | EscapedChar;
UnescapedChar = ? 除了 " 和 \ 的任何字符 ?;
EscapedChar = "\", ("\" | '"');

(* 变量引用 *)
VariableRef = "${", VariableName, "}";
VariableName = Identifier;

(* 空白字符 *)
whitespace = {" " | "\t" | "\r" | "\n"};

(* 外部模块引用 *)
ExternModule = "extern", whitespace, "mod", whitespace, ModuleNameList, whitespace, "{", whitespace, ModuleSource, whitespace, "}", whitespace, ";";
ModuleNameList = ModuleName, {",", whitespace, ModuleName};
ModuleSource = PathSource | GitSource;
PathSource = "path", whitespace, "=", whitespace, String;
GitSource = "git", whitespace, "=", whitespace, String, whitespace, ",", whitespace, "channel", whitespace, "=", whitespace, String;


(* 注解 *)
Annotation = "#[", AnnotationName, ("(", AnnotationParams, ")")?, "]";
AnnotationName = Identifier;
AnnotationParams = AnnotationParam, {",", whitespace, AnnotationParam};
AnnotationParam = Identifier, whitespace, "=", whitespace, String;
```




## 示例
``` rust
env dev {
	root = "${HOME}/my_project";
	gx.read_cmd {
		name = "MY_PATH" ;
		cmd  = "pwd" ;
	};
}
```

``` rust
mod my_module {
    -- 模块属性
    author = "John Doe";
    version = "1.0";

    -- 环境定义
    env test {
        root = "${HOME}/test_project";
        gx.read_cmd {
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
