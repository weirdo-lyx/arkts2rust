# STEP 6：函数声明 + 基础类型标注 + CodeGen

本 Step 的目标：支持在顶层声明函数，并给函数参数/返回值加上（可选的）基础类型标注，然后把它们翻译成可编译的 Rust `fn`。

核心闭环变成：

```text
ArkTS 源码
  -> Lexer(Token)
  -> Parser(AST: Program{funcs, stmts})
  -> CodeGen(Rust 源码：先 fn 声明，再 fn main)
```

---

## 1. 支持的语法

新增支持（Step6）：

```text
function name(a: number, b: number): number { ... }
function name(a, b) { ... }             // 类型标注可省略
```

约束：
- 参数/返回类型只支持：`number | string | boolean | void`
- 类型标注是 **可选** 的，但仅用于 codegen，不做完整类型推导/检查
- 顶层可以混排：多个函数声明 + 顶层语句
  - 顶层语句会被放进 Rust 的 `fn main() { ... }`

仍然不支持（本 Step 明确不做）：
- 闭包
- 泛型
- import/export
- function 的嵌套声明（只允许在顶层声明）

---

## 2. 类型映射（ArkTS -> Rust）

| ArkTS | Rust |
|---|---|
| number | i32 |
| string | String |
| boolean | bool |
| void | ()（在 Rust 里通常省略返回类型） |

### 2.1 重要说明：类型标注只用于 codegen

本项目在 Step6 **没有实现完整类型系统**，因此：
- 不会做类型推导（例如从表达式推断参数类型）
- 不会做类型检查（例如 `return "x"` 但写了 `: number`）

CodeGen 只是把标注“按字面”映射到 Rust 类型上。

### 2.2 当类型标注省略时怎么生成？

为了让代码还能生成可编译的 Rust，本项目采用简单默认规则：
- 参数类型省略：默认 `i32`
- 返回类型省略：
  - 如果函数体中出现 `return <expr>;`（带返回值），默认 `i32`
  - 如果只有 `return;` 或根本没有 return，默认 `void`（Rust 里就是不写 `-> ...`）

这不是类型推导，只是“为了可生成”的默认值策略。

---

## 3. AST 结构变化

见 `src/ast.rs`：
- `Program` 新增 `funcs: Vec<FuncDecl>` 与 `stmts: Vec<Stmt>`
- 新增：
  - `FuncDecl { name, params, ret_type, body }`
  - `Param { name, ty }`
  - `TypeAnn::{Number,String,Boolean,Void}`

---

## 4. Parser 变化

见 `src/parser/parser.rs`：
- 顶层循环解析时：
  - 遇到 `function` -> 解析 `FuncDecl` 放入 `Program.funcs`
  - 否则按语句解析 -> 放入 `Program.stmts`
- 参数列表支持 `a: number` 这种形式（需要 `:` Token）
- 函数体必须是 block：`{ stmt* }`，否则报 `ExpectedBlock`

---

## 5. CodeGen 变化（先 fn 声明，再 main）

见 `src/codegen.rs`：
- 生成顺序：
  1) 所有顶层函数 `fn ... { ... }`
  2) `fn main() { ... }`（装载顶层语句）
- `return` 的处理：
  - 函数返回值类型是 `void` 时，`return expr;` 会被翻译为：

```rust
let _ = expr;
return;
```

  - 函数返回值类型不是 `void` 时，`return;` 会报错 `ReturnValueRequired`

---

## 6. 测试与验证

新增测试：`tests/function_tests.rs`，覆盖：
- 函数声明解析（含类型/不含类型）
- 函数调用
- return / void 的边界行为
- 错误用例（UnknownType、ExpectedBlock、ReturnValueRequired）
- 生成 Rust 并调用 `rustc` 编译验证（若本机没有 rustc 会跳过）

运行：

```bash
cargo test
```

---

## 7. 手动验证生成 Rust 可编译

1) 写一个 `input.ets`：

```text
function add(a: number, b: number): number { return a + b; }
add(1, 2);
return;
```

2) 生成 `output.rs`：

```bash
cargo run -- input.ets -o output.rs
```

3) 编译：

```bash
rustc output.rs
```

---

## 8. 论文段落（可直接引用/改写）

为提升编译器子集语言的模块化能力，本文在控制流与表达式支持的基础上引入顶层函数声明，并支持参数与返回值的基础类型标注。语法分析阶段对顶层结构进行分区：函数声明被解析为独立的 AST 节点集合，而顶层可执行语句统一放入主入口以适配 Rust 的执行模型。代码生成阶段将 ArkTS 的 `function` 映射为 Rust 的 `fn`，并根据类型标注完成 `number/string/boolean/void` 到 `i32/String/bool/()` 的直接映射。需要强调的是，类型标注仅用于代码生成而非完整类型系统；本文不实现泛型、闭包及模块导入导出等特性。通过覆盖函数声明、调用、返回语义与错误用例的测试，并使用 `rustc` 对生成代码进行可编译性验证，证明该扩展在最小闭环下具备正确性与可维护性。

