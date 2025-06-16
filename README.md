#  Galaxy Flow 

Galaxy Flow 是基于环境和工作流组织的DevSecOps领域专用语言，专注于自动化安全流程的编排与管理。

##  下载

项目的正式发布版本可在GitHub发布页面获取：

https://github.com/galaxy-sec/galaxy-flow/releases

## 语言规范
### 文件扩展
```
.gxl
```

## 命令行工具

### 核心命令
```
gx
gm
```

#### gx
对项目定义的工作流（ work.gxl） 运行

#### gm
对项目定义的管理流（ adm.gxl） 运行

### 任务返回结果配置
1. 在环境变量中设置task_id，例如：`export task_id='1234567890'`设置父id
task_report, 例如：`export task_report='http://42.194.144.213:8080/task/create_batch_subtask/'`设置任务上报返回地址
task_result， 例如：`export task_result='http://42.194.144.213:8080/task/update_subtask_info/'`设置任务结果返回地址
2. 在_gal文件夹下设置task_config.toml文件，例如：
```
[task]
task_report = 'http://42.194.144.213:8080/task/create_batch_subtask/'
task_result = 'http://42.194.144.213:8080/task/update_subtask_info/'
```
task_id任然使用环境变量设置
