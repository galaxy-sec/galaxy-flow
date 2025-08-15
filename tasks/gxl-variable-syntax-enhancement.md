# GXL变量语法增强设计方案

## 背景

当前GXL语言中，变量引用严格使用`${var}`语法。这种语法虽然明确，但在某些场景下显得冗长，用户体验不够友好。本方案旨在解除这一限制，同时保持向后兼容性。

## 现状分析

### 当前实现
- **语法规则**: 严格使用`${variable_name}`格式
- **解析位置**: 主要在parser模块的`atom.rs`和`context.rs`中处理
- **使用场景**: 变量替换、模板渲染、配置引用

### 代码位置
- `src/parser/atom.rs` - 原子值解析
- `src/parser/context.rs` - 上下文变量解析
- `src/evaluator/` - 变量求值

## 设计方案

### 目标
1. **简化语法**: 支持更简洁的变量引用方式
2. **向后兼容**: 保持`${var}`语法继续有效
3. **渐进增强**: 新语法与旧语法共存

### 新语法规则

#### 1. 简化变量引用
```
# 当前语法（继续支持）
${variable_name}

# 新增语法选项
$variable_name      # 简化版本，适用于简单变量名
{variable_name}     # 无$前缀版本
variable_name       # 裸变量名（需要上下文判断）
```

#### 2. 作用域规则
- **全局变量**: 支持所有新语法
- **局部变量**: 优先使用新语法，回退到旧语法
- **环境变量**: 保持`${ENV_VAR}`格式不变

#### 3. 解析优先级
1. 精确匹配：`${exact_name}`（最高优先级）
2. 简化语法：`$simple_name`
3. 大括号语法：`{name}`
4. 裸变量名：`name`（最低优先级，需要上下文验证）

### 实现方案

#### 阶段1: 语法扩展（parser层）

**文件**: `src/parser/atom.rs`

```rust
// 新增语法模式
enum VariableSyntax {
    Braced,      // ${var}
    Dollar,      // $var
    Curly,       // {var}
    Bare,        // var
}

struct VariableRef {
    name: String,
    syntax: VariableSyntax,
    span: Span,
}
```

#### 阶段2: 解析器增强

**文件**: `src/parser/context.rs`

```rust
impl ContextParser {
    fn parse_variable_ref(&mut self) -> Result<VariableRef, ParseError> {
        // 支持多种语法格式
        if self.consume("${") {
            // 传统语法
            self.parse_braced_variable()
        } else if self.consume("$") {
            // 简化语法
            self.parse_dollar_variable()
        } else if self.consume("{") {
            // 大括号语法
            self.parse_curly_variable()
        } else {
            // 裸变量名（需要上下文）
            self.parse_bare_variable()
        }
    }
}
```

#### 阶段3: 求值器适配

**文件**: `src/evaluator/mod.rs`

```rust
impl Evaluator {
    fn evaluate_variable(&self, var_ref: &VariableRef, context: &Context) -> Result<Value, Error> {
        match var_ref.syntax {
            VariableSyntax::Braced => {
                // 传统精确匹配
                context.get_variable(&var_ref.name)
            }
            VariableSyntax::Dollar => {
                // 简化语法，支持默认值
                context.get_variable_simplified(&var_ref.name)
            }
            VariableSyntax::Curly => {
                // 大括号语法，支持表达式
                context.evaluate_curly_expr(&var_ref.name)
            }
            VariableSyntax::Bare => {
                // 裸变量名，需要验证存在性
                context.resolve_bare_variable(&var_ref.name)
            }
        }
    }
}
```

### 兼容性策略

#### 1. 配置开关
```toml
[gxl.syntax]
# 语法兼容性设置
enable_simplified_syntax = true
enable_bare_variables = true
legacy_mode = false  # 设为true时只支持旧语法
```

#### 2. 迁移指南
- **现有代码**: 无需修改，继续工作
- **新代码**: 推荐使用简化语法
- **混合使用**: 同一文件中可同时使用新旧语法

#### 3. 警告机制
```rust
// 在解析时提供兼容性警告
if self.legacy_mode && !matches!(syntax, VariableSyntax::Braced) {
    self.warn("使用非传统变量语法，可能影响旧版本兼容性");
}
```

### 测试策略

#### 单元测试
- `tests/syntax_compatibility.rs` - 兼容性测试
- `tests/variable_parsing.rs` - 语法解析测试
- `tests/mixed_syntax.rs` - 混合语法测试

#### 集成测试
- 现有测试用例继续通过
- 新增简化语法测试用例
- 性能基准测试（确保无性能退化）

### 风险与缓解

#### 主要风险
1. **歧义解析**: 裸变量名可能与字符串冲突
2. **性能影响**: 多种语法格式可能降低解析速度
3. **兼容性**: 旧版本GXL运行时可能无法识别新语法

#### 缓解措施
1. **严格验证**: 裸变量名必须存在于变量表中
2. **缓存机制**: 解析结果缓存，减少重复解析开销
3. **版本标记**: 在文件头部添加语法版本声明

### 实施计划

#### 第1周: 基础框架
- [ ] 设计新的语法结构
- [ ] 实现parser层扩展
- [ ] 添加配置选项

#### 第2周: 求值器适配
- [ ] 修改evaluator模块
- [ ] 实现上下文解析
- [ ] 添加兼容性检查

#### 第3周: 测试与优化
- [ ] 编写全面测试用例
- [ ] 性能基准测试
- [ ] 文档更新

#### 第4周: 发布准备
- [ ] 用户文档更新
- [ ] 迁移指南编写
- [ ] 版本发布

### 验证标准

1. **功能验证**: 所有新语法格式正确工作
2. **兼容性验证**: 现有`${var}`语法100%兼容
3. **性能验证**: 解析性能不低于现有实现
4. **测试验证**: 测试覆盖率保持在90%以上

### 相关文件变更

```
src/parser/atom.rs      # 新增语法解析
src/parser/context.rs   # 变量引用解析
src/evaluator/mod.rs    # 求值逻辑适配
src/conf/gxlconf.rs     # 配置选项添加
tests/                  # 新增测试文件
```

## 结论

本方案通过渐进式增强的方式，在保持完全向后兼容的前提下，为GXL变量语法提供了更灵活、更简洁的表达方式。通过分阶段实施和全面的测试策略，确保系统的稳定性和可靠性。