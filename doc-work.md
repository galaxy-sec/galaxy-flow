# 文档工作

## 文档输出为 docs 目录

## 规则
* gxl 的代码，使用 rust 进代码块的标记， 以支持语法高亮

## 任务

### 整理 内建的能力

* 输入： src/parser/inner
* 输出： docs/inner
* 任务： 遍历 src/parser/inner目录, 代码生成markdown文档, 包括语法定义和示例代码, 每一种能力单独输出一个文件

## 任务状态

* 已完成：gx.assert, gx.cmd, gx.echo, gx.read, gx.vars, gx.tpl, gx.ver, gx.shell, gx.download/gx.upload, gx.tar/gx.untar, gx.run, gx.defined 的文档生成
* 文档已保存至 docs/inner/index.md
* 每种能力的单独文档已生成至 docs/inner/ 目录下的独立文件中

### 整理 Example 代码到文档

* 输入： examples/
* 输出： docs/example/
* 任务： 从 examples/ 目录下的独立文件中提取示例代码, 输出到 docs/example/ 目录下
* 状态： 已完成
