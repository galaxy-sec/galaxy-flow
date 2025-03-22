# Galaxy Flow Lang 



## 解决什么问题?

 为Shell 脚本提供现代化的封装，它以环境、工作流为中心进行组织。


##  概念



### env
环境（env）用于定义系统运行时的环境配置，包括属性（Property）和动作调用（ActCall）。
- **Property**：属性用于存储环境的配置信息，如路径、变量等。
- **ActCall**：动作调用用于执行特定的操作，如读取文件、执行命令等。

#### 示例
```
env dev {
	root = "${HOME}/my_project" ;
	gx.read {
		name = "MY_PATH" ;
		cmd  = "pwd" ;
	};
}
```



混合 env

示例:

```
// dev 混合 base 和 java 的env ;
env dev : base, java
```



### flow

* Property
* ActCall
#### 示例
```
flow my_flow {
	step1 = "execute_task";
	task1.run {
		param1 = "value1";
		param2 = "value2";
	};
}
```

### mod
模块（mod）是 Galaxy Flow Lang 中的一个重要概念，用于组织和管理代码。一个模块可以包含多个环境（env）和流程（flow）定义。

## 目标是什么领域?

Galaxy Flow Lang 的目标领域是系统开发和运行领域，该领域具有以下特点：
### 环境相关
系统的运行环境可能包括操作系统、硬件平台、网络环境等，不同的环境可能需要不同的配置和操作。



### 安全密钥管理
在系统开发和运行过程中，需要对安全密钥进行管理，确保系统的安全性。




## 结构

### mod
模块（mod）是 Galaxy Flow Lang 中的一个重要概念，用于组织和管理代码。一个模块可以包含多个环境（env）和流程（flow）定义。

### env
环境（env）用于定义系统运行时的环境配置，包括属性（Property）和动作调用（ActCall）。
- **Property**：属性用于存储环境的配置信息，如路径、变量等。
- **ActCall**：动作调用用于执行特定的操作，如读取文件、执行命令等。

#### 示例
```
env dev {
	root = "${HOME}/my_project" ;
	gx.read {
		name = "MY_PATH" ;
		cmd  = "pwd" ;
	};
}
```



混合 env

示例:

```
// dev 混合 base 和 java 的env ;
env dev : base, java
```



## flow

* Property
* ActCall
