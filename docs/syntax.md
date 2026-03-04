
# GXL 文件语法

## GXL 语法定义
GXL（Galaxy Flow Language）是一种为 DevSecOps 自动化工作流设计的领域特定语言。根据代码库中的信息，GXL 的语法定义主要在 galaxy-sec/galaxy-flow 仓库的解析器模块中实现。

### GXL 语法结构
GXL 语言遵循模块化、层次化的结构，主要由以下核心组件组成：

* 模块（Modules）：使用 mod 关键字定义，是 GXL 的顶层组织单元
* 环境（Environments）：使用 env 关键字定义，用于配置不同的执行上下文
* 流程（Flows）：使用 flow 关键字定义，表示工作流程序
* 函数（Functions）：使用 fn 关键字定义，可重用的函数单元
* 活动（Activities）：使用 activity 关键字定义，可重用的操作单元，在模块级别定义


## EBNF 语法
```EBNF

(* GXL 文件由一系列模块定义组成 *)
GXL-File = {Module};

(* 模块定义 *)
Module = [Annotation], "mod", whitespace, ModuleName, [whitespace, ":", whitespace, ModuleRefList], whitespace, "{", whitespace, ModuleContent, whitespace, "}", whitespace, ";";
ModuleName = Identifier;
ModuleRefList = ModuleRef, {",", whitespace, ModuleRef};
ModuleRef = Identifier;
ModuleContent = {Property | Environment | Flow | Function | Activity};

(* 属性定义 (键值对) *)
Property = PropertyName, whitespace, "=", whitespace, PropertyValue, whitespace, ";";
PropertyName = Identifier;
PropertyValue = String;

(* 环境定义 *)
Environment = [Annotation], "env", whitespace, EnvName, [whitespace, ":", whitespace, EnvRefList], whitespace, "{", whitespace, EnvContent, whitespace, "}", whitespace, ";";
EnvName = Identifier;
EnvContent = {Property | EnvCommand};
EnvRefList = EnvRef, {",", whitespace, EnvRef};
EnvRef = Identifier;

(* 流程定义 - 定义形式 *)
Flow = [Annotation], (DefinitionFlow | ReferenceFlow);

(* 定义流程（管道分隔）*)
DefinitionFlow = "flow", whitespace, [FlowRefList, whitespace, "|"], whitespace, "@", FlowName, [whitespace, "|", whitespace, FlowRefList], whitespace, "{", whitespace, FlowContent, whitespace, "}", whitespace, ";";

(* 引用其他流程 *)
ReferenceFlow = "flow", whitespace, FlowName, whitespace, ":", whitespace, FlowRefList, whitespace, ";";
FlowName = Identifier;
FlowRefList = FlowRef, {",", whitespace, FlowRef};
FlowRef = Identifier;
FlowContent = {Command | Property};

(* 函数定义 *)
Function = [Annotation], "fn", whitespace, FunctionName, whitespace, "(", whitespace, [FunctionParams], whitespace, ")", whitespace, "{", whitespace, FunctionContent, whitespace, "}", whitespace, ";";
FunctionName = Identifier;
FunctionParams = FunctionParam, {",", whitespace, FunctionParam};
FunctionParam = ["*"], Identifier, [whitespace, "=", whitespace, String];
FunctionContent = {Command};

(* 活动定义 *)
Activity = [Annotation], "activity", whitespace, ActivityName, whitespace, "{", whitespace, ActivityContent, whitespace, "}", whitespace, ";";
ActivityName = Identifier;
ActivityContent = {Property};

(* 命令定义 *)
Command = (BuiltinCommand | ActivityCall), whitespace, ";";

(* 内置命令 - 使用函数调用语法 *)
BuiltinCommand = "gx.", CommandName, whitespace, "(", whitespace, CommandProps, whitespace, ")";
CommandName = "echo" | "vars" | "cmd" | "read" | "tpl" | "assert" | "ver";
CommandProps = {PropertyAssignment}, {",", whitespace, PropertyAssignment};
PropertyAssignment = PropertyName, whitespace, ":", whitespace, PropertyValue;

(* 环境专用命令 *)
EnvCommand = "gx.vars", whitespace, "{", whitespace, CommandProps, whitespace, "}"
           | "gx.read_cmd", whitespace, "(", whitespace, CommandProps, whitespace, ")"
           | "gx.read_stdin", whitespace, "(", whitespace, CommandProps, whitespace, ")"
           | "gx.read_file", whitespace, "(", whitespace, CommandProps, whitespace, ")";

(* 活动调用 *)
ActivityCall = ActivityName, whitespace, "(", whitespace, CommandProps, whitespace, ")";
ActivityName = Identifier, {".", Identifier};

(* 标识符 *)
Identifier = Alpha, {Alpha | Digit | "_"};
Alpha = "A" | "B" | ... | "Z" | "a" | "b" | ... | "z";
Digit = "0" | "1" | ... | "9";

(* 字符串 *)
String = '"', {StringChar}, '"';
StringChar = UnescapedChar | EscapedChar;
UnescapedChar = ? 除了 " 和 \ 的任何字符 ?;
EscapedChar = "\\", ("\\" | '"');

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
Annotation = "#[", AnnotationName, ["(", AnnotationParams, ")"], "]";
AnnotationName = Identifier;
AnnotationParams = AnnotationParam, {",", whitespace, AnnotationParam};
AnnotationParam = Identifier, whitespace, "=", whitespace, String;
```




## 实现代码对应关系

### 1. 模块解析实现
**EBNF**: `Module = [Annotation], "mod", ...`
**实现位置**: <mcfile name="stc_mod.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_mod.rs"></mcfile>
**主要函数**: <mcsymbol name="gal_stc_mod" filename="stc_mod.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_mod.rs" startline="40" type="function"></mcsymbol>
**功能**: 解析模块定义，支持注解、模块引用列表、属性和环境/流程/函数/活动内容

### 2. 环境解析实现
**EBNF**: `Environment = [Annotation], "env", ...`
**实现位置**: <mcfile name="stc_env.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_env.rs"></mcfile>
**主要函数**: <mcsymbol name="gal_stc_env" filename="stc_env.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_env.rs" startline="48" type="function"></mcsymbol>
**功能**: 解析环境定义，支持注解、环境引用列表、属性和环境专用命令

### 3. 流程解析实现
**EBNF**: `Flow = [Annotation], (DefinitionFlow | ReferenceFlow)`
**实现位置**: <mcfile name="stc_flow/head.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_flow/head.rs"></mcfile>
**主要函数**: <mcsymbol name="galaxy_flow_head" filename="head.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_flow/head.rs" startline="15" type="function"></mcsymbol>
**功能**: 解析流程头部，支持两种语法形式（定义、引用流程）
**流程体解析**: <mcsymbol name="gal_stc_flow_body" filename="body.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/stc_flow/body.rs" startline="11" type="function"></mcsymbol>

### 4. 基础元素解析实现
**EBNF**: `Identifier`, `String`, `VariableRef`
**实现位置**: <mcfile name="atom.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/atom.rs"></mcfile>
**主要函数**: 
- <mcsymbol name="take_var_ref_name" filename="atom.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/atom.rs" startline="47" type="function"></mcsymbol> - 解析变量引用
- <mcsymbol name="take_dot_pair" filename="atom.rs" path="/Users/zuowenjian/devspace/galaxy/galaxy-flow/src/parser/atom.rs" startline="30" type="function"></mcsymbol> - 解析点分隔标识符


## 示例
```gxl
env dev {
    root = "${HOME}/my_project";
    gx.read_cmd (
        name : "MY_PATH",
        cmd  : "pwd"
    );
}
```

```gxl
#[author("John Doe")]
#[version("1.0")]
mod my_module : mod_a, mod_b {
    # 模块属性
    author = "John Doe";
    version = "1.0";

    # 环境定义
    env test {
        root = "${HOME}/test_project";
        gx.read_cmd (
            name : "TEST_PATH",
            cmd  : "ls"
        );
    }

    # 函数定义
    fn echo_message(msg) {
        gx.echo (value : msg);
    }

    # 活动定义
    activity task_runner {
        timeout = "30s";
        retry_count = 3;
    }

    # 流程定义 - 定义流程
    flow before_flow | @my_flow | after_flow {
        gx.echo (value : "Hello from my_flow");
        task_runner.run (
            param1 : "value1",
            param2 : "value2"
        );
    }

    # 流程定义 - 定义流程（管道分隔）
    flow before_flow | @main_flow | after_flow {
        gx.echo (value : "Pipeline flow execution");
    }

    # 引用流程
    flow reference_flow : my_flow, other_flow;
}
```

### 外部模块引用示例
```gxl
extern mod mod_a { path = "@{PATH}"; }
extern mod mod_b { 
    git = "https://github.com/example/repo.git", 
    channel = "main" 
}
```
