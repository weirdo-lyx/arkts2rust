# STEP 5：控制流语句（Block / If / While / Return）+ CodeGen

本 Step 的目标：在 Step4 表达式的基础上，加入最常用的控制流语句，并把它们生成成等价的 Rust 代码，继续保持“最小闭环”。

---

## 1. 本 Step 支持的语法

新增支持：
- block：`{ stmt* }`
- if/else：`if (cond) stmt else stmt`
- while：`while (cond) stmt`
- return：`return expr? ;`

仍然禁止（留到 Step6 之后）：
- function 声明
- if/while 之外的更多语句

---

## 2. 重要限制：条件必须是 bool（不支持 JS truthy）

为了让生成的 Rust 代码语义明确、可编译，本 Step 做了一个强约束：

> `if (...)` 和 `while (...)` 的条件表达式必须是 bool。  
> 不支持 JavaScript/TypeScript 那种 “truthy/falsy”（例如 `if (1)`、`if ("x")`）。

### 2.1 什么是 truthy（本项目不支持）

在 JS/TS 里：
- `if (1)` 会被当作 true
- `if ("hello")` 会被当作 true
- `if (0)` 会被当作 false

但 Rust 里 **只有 bool 才能当条件**，所以我们必须拒绝这些写法。

### 2.2 Parser 的“保守检查”策略

我们没有做完整类型系统，所以 Parser 只能做“看起来像 bool”的检查：
- 明确不是 bool：`1`、`"x"`、`1+2`、`-1` 这类直接报错
- 允许：比较/相等/逻辑运算（例如 `x < 3`、`a && b || c`）
- 允许：标识符/函数调用（例如 `if (flag)`、`if (is_ok())`），因为它们可能是 bool（类型未知时不强行拒绝）

---

## 3. AST 改动（新增语句节点）

见 `src/ast.rs`，新增：
- `Stmt::Block(BlockStmt)`：`{ stmt* }`
- `Stmt::If(IfStmt)`：`if (cond) ... else ...`
- `Stmt::While(WhileStmt)`：`while (cond) ...`
- `Stmt::Return(ReturnStmt)`：`return expr?;`

---

## 4. Parser 改动（支持嵌套 block）

见 `src/parser/parser.rs`。

关键点：
- `parse_stmt()` 新增分支识别 `{` / `if` / `while` / `return`
- `parse_block_stmt()` 内部通过循环持续解析 `stmt`，直到遇到 `}`
- if/while 的条件解析仍然复用 Step4 的优先级解析（Pratt），然后额外做“bool 条件”检查

---

## 5. CodeGen 改动（生成 Rust 控制流）

见 `src/codegen.rs`。

映射策略：
- block：直接生成 Rust 块 `{ ... }`
- if/else：生成 Rust `if { ... } else { ... }`
- while：生成 Rust `while { ... }`
- return：
  - `return;` -> `return;`
  - `return expr;`：由于我们生成的函数签名固定为 `fn main() { ... }`（返回 `()`），
    Rust 里不能直接 `return <expr>;`。因此 CodeGen 采用“提前结束 + 丢弃返回值”的处理：

```rust
let _ = <expr>;
return;
```

这样可以保证生成的 Rust 程序可编译，同时保持“提前退出”的行为一致。

---

## 6. 测试

新增控制流测试文件：`tests/control_flow_tests.rs`，包含：
- if/else 用例 ≥6
- while 用例 ≥4
- 错误用例 ≥2（MissingElse / ConditionMustBeBool 等）
- 额外包含一个用例：把生成的 Rust 写入临时文件并调用 `rustc` 编译（如果本机没有 rustc 会跳过）

运行：

```bash
cargo test
```

---

## 7. 手动验证生成 Rust 可编译（推荐）

1) 写一个 `input.ets`：

```text
let x=0;
while (x<3) { x=x+1; }
if (x==3) { console.log("ok"); } else { console.log("bad"); }
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

为提高编译器子集语言的表达能力，本文在表达式解析的基础上扩展了控制流语句，包括代码块、条件分支、循环与返回语句。语法分析阶段支持嵌套块结构，并在 if/while 条件处引入布尔约束以避免 JavaScript/TypeScript 的 truthy/falsy 语义，从而确保生成 Rust 代码具有明确且可编译的条件表达式。代码生成阶段将控制流节点映射为 Rust 的 `if/else`、`while` 与块语法，并对返回语句采用“提前退出”的策略以适配 `fn main()` 的返回类型。通过覆盖典型分支/循环场景的单元测试与错误用例验证，保证了控制流扩展后的解析与生成在最小闭环中的正确性与可维护性。

