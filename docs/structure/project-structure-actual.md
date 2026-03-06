# Galaxy Flow 项目实际结构

本文档仅描述当前仓库中的真实结构（以当前工作树代码为准）。

## Workspace 主体

```text
galaxy-flow/
├── app/
│   ├── gflow/main.rs        # gflow CLI 入口
│   └── gprj/{main,args}.rs  # gprj CLI 入口与参数
├── src/
│   ├── ability/             # GXL abilities 实现
│   ├── calculate/           # 表达式与条件计算
│   ├── conf/                # 项目配置加载
│   ├── evaluator/           # 环境表达式渲染
│   ├── model/               # 运行时与语法模型
│   ├── parser/              # GXL 语法解析
│   ├── self_update/         # 自升级检查/安装/状态存储
│   └── util/                # 通用工具
├── crates/
│   ├── orion_parse/         # 解析基础能力
│   └── orion_cond/          # 条件表达式能力
├── docs/
├── examples/
├── tests/
└── updates/                 # stable/alpha/beta 更新清单
```

## `src` 模块导出概览

- `ability`: `ai, archive, assert, cmd, delegate, echo, gxl, load, patch, read, shell, tpl, version`
- `calculate`: `compare, cond, defined, dynval, express, logic, traits`
- `conf`: `gxlconf, oprator`（`mod_test` 为内部测试模块）
- `evaluator`: 对外仅导出 `EnvExpress, VarParser`（来自 `env_exp.rs`）
- `model`: `annotation, components, context, data, error, execution, expect, meta, primitive, task_report, traits, var`
- `parser`: `atom, domain, externs, abilities, cond, context, gxl_fun, inner, stc_*`
- `self_update`: `client, installer, model, service, storage`
- `util`: `git(内部), http_handle, path, shell, accessor, diagnose, redirect` 等

## 对齐文档

- `docs/structure/ability-actual.md`
- `docs/structure/calculate-actual.md`
- `docs/structure/conf-actual.md`
- `docs/structure/evaluator-actual.md`
- `docs/structure/model-actual.md`
- `docs/structure/parser-actual.md`
- `docs/structure/util-actual.md`

## 维护规则

- 本目录中，`*-actual.md` 是结构事实文档。
- 同名 `*.md` 仅保留简要入口说明，避免双份内容漂移。
