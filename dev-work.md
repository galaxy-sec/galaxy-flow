# 开发计划

## 工作规则
- 工作任务结束，要把反馈写到此文档中。
- 对于设计型的任务，需要先写出设计方案文档，放置到tasks 目录下。
- 任务的方案，经过评审后，才能开始实现。


## 任务列表

### 分析当前程序结构
 [x] 为每个模块，编写一个结构文档。文档中用图表达结构之前的关系。

**完成详情：**
- ✅ 基于实际代码结构，修正了之前文档中的虚构内容
- ✅ 创建了7个核心模块的实际结构文档，保存在 `docs/structure/` 目录
- ✅ 文档包含：模块概述、实际文件结构、核心组件说明、依赖关系图
- ✅ 创建了项目整体结构总览文档

**实际创建的文档：**
- `docs/structure/ability-actual.md` - ability模块实际结构
- `docs/structure/calculate-actual.md` - calculate模块实际结构  
- `docs/structure/conf-actual.md` - conf模块实际结构
- `docs/structure/evaluator-actual.md` - evaluator模块实际结构
- `docs/structure/model-actual.md` - model模块实际结构
- `docs/structure/parser-actual.md` - parser模块实际结构
- `docs/structure/util-actual.md` - util模块实际结构
- `docs/structure/project-structure-actual.md` - 项目整体结构总览

所有文档均基于实际代码文件结构，无虚构内容。

### GXL 对于变量的引用都通过语法${var}来实现，在代码中解除这个语法的限制，但需要兼容
[x] 设计出方案，放置到tasks 目录下。

**完成详情：**
- ✅ 分析了当前变量语法实现的代码位置（parser/atom.rs, parser/context.rs, evaluator/mod.rs）
- ✅ 设计了渐进式语法增强方案，支持$var、{var}、var等新语法
- ✅ 确保100%向后兼容，现有${var}语法继续有效
- ✅ 制定了完整的实施计划（4周分阶段实施）
- ✅ 包含详细的测试策略和风险缓解措施

**方案文档：**
- `tasks/gxl-variable-syntax-enhancement.md` - 完整的设计方案文档
- 包含：背景分析、语法规则、实现方案、兼容性策略、测试计划、实施时间表