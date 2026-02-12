# STEP 2：语法分析（Parser）——把 Token 流变成 AST

本 Step 的目标：把 Step1 的 `Vec<Token>` 按照一套极小的语法规则“组装”成 AST（抽象语法树），并在遇到错误时给出带 `Span` 的定位信息。

## 1. 什么是语法分析（Parser）

词法分析（Lexer）负责把字符切成 Token，例如：

```text
let x = 1;
```

会变成：

```text
KwLet Ident("x") Eq Number(1) Semicolon
```

语法分析（Parser）负责理解这些 Token 的结构关系：这是一条“变量声明语句”，变量名是 `x`，初始值是字面量 `1`，并且必须以 `;` 结束。

## 2. 本 Step 支持的最小语法（产生式）

```text
Program  = { Stmt }

Stmt     = VarDecl ";"
         | ExprStmt ";"

VarDecl  = ("let" | "const") Ident "=" Literal

ExprStmt = CallExpr

CallExpr = "console" "." "log" "(" Literal ")"

Literal  = Number | String | "true" | "false"
```

限制（硬性约束）：
- 每条语句必须以分号 `;` 结束
- `let/const` 只允许初始化为 literal（不支持 `let x = y;`）
- 表达式语句只支持 `console.log(literal);`
- 不实现复杂表达式与优先级（比如 `1 + 2 * 3` 这种都不支持）

## 3. AST 设计（为什么需要 AST）

AST 是“对语法结构的树形表达”，它是后续步骤（类型检查、代码生成等）的输入。

本 Step 的 AST（见 `src/ast.rs`）核心结构：
- `Program { stmts: Vec<Stmt> }`：整个文件
- `Stmt`：
  - `VarDecl`：`let/const` 声明
  - `ExprStmt`：表达式语句
- `Expr`：
  - `Literal(...)`
  - `Call(...)`（目前只有 `console.log`）
- `Literal`：`Number(i32)` / `String(String)` / `Bool(bool)`

## 4. 常见错误与 Span 定位

Parser 会对“缺少关键 Token”的情况报错，并把错误定位到当前 token 的 `Span`（如果已经到 EOF，就使用最后一个 token 的 `Span`）。

本 Step 重点覆盖：
- `MissingSemicolon`：语句末尾缺 `;`
- `MissingRParen`：`console.log(...)` 缺 `)`
- `UnknownStructure`：遇到不在子集里的结构（例如 `foo(1);`）
- `ExpectedLiteral`：需要 literal 的地方给了别的（例如 `console.log(x);`）

## 5. 如何运行测试

```bash
cargo test
```

本 Step 新增测试文件：`tests/parser_tests.rs`（包含正确用例与错误用例）。

## 6. 论文段落（可直接引用/改写）

在源到源编译器的前端阶段，语法分析负责将词法分析产生的 Token 序列还原为具有层次结构的抽象语法树（AST）。相比线性的 Token 流，AST 能显式表达语句与表达式的嵌套关系，为后续的语义分析与代码生成提供统一的数据结构表示。本文实现了一个面向 ArkTS 子集的递归下降解析器，在最小语法集（变量声明与特定调用表达式）上验证了从源代码到 AST 的可行性，并通过携带 Span 的错误报告机制提升了语法错误定位的可用性。

