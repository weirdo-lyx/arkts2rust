# Step 0：Cargo 工程骨架（占位实现）

## 本 Step 新增了什么
- 创建 Rust Cargo 项目最小骨架（library + binary）。
- 建立固定的工程化目录结构与模块边界（error/span/lexer/parser/ast/codegen）。
- 提供 `compile` 占位 API 与最简 CLI 桩。
- 提供 1 个 smoke test，确保 `cargo test` 可稳定通过。

## 本 Step 允许修改的文件白名单
- `Cargo.toml`
- `src/main.rs`
- `src/lib.rs`
- `src/error.rs`
- `src/span.rs`
- `src/lexer/mod.rs`
- `src/lexer/token.rs`
- `src/lexer/lexer.rs`
- `src/parser/mod.rs`
- `src/parser/parser.rs`
- `src/ast.rs`
- `src/codegen.rs`
- `tests/step0_smoke.rs`
- `docs/STEP_0.md`

## 每个文件做什么
- `src/span.rs`：定义 `Span { start, end }`，用于未来错误定位与 AST/Token 标注。
- `src/error.rs`：定义统一错误类型 `Error { code, span }` 与构造函数 `Error::new(...)`。
- `src/lexer/*`：词法分析模块的占位定义（Token 类型与 `lex` 函数签名）；本 Step 不实现任何切词规则。
- `src/parser/*`：语法分析模块的占位定义（`parse` 函数签名）；本 Step 不实现任何语法规则。
- `src/ast.rs`：AST 的最小占位结构（`Program/Stmt`）。
- `src/codegen.rs`：代码生成模块占位定义（`generate` 函数签名）；本 Step 不实现任何生成逻辑。
- `src/lib.rs`：对外 API `pub fn compile(src: &str) -> Result<String, Error>`（占位返回 `NotImplemented`）。
- `src/main.rs`：最简 CLI：读取 input 文件，调用 `compile`，写 output 文件；若失败打印清晰错误。
- `tests/step0_smoke.rs`：smoke test，验证 `compile` 当前为占位错误但可调用。


### 1. 项目配置与对外入口

#### [Cargo.toml](file:///Users/lyx/research/arkts2rust/Cargo.toml)
这是 Rust 项目的清单文件。
- **依赖管理**：目前我们保持极简，没有引入任何运行时依赖（符合你的要求）。
- **Dev-dependencies**：虽然现在也是空的，但未来如果需要测试辅助库（如 `insta` 做快照测试），会加在这里。

#### [src/lib.rs](file:///Users/lyx/research/arkts2rust/src/lib.rs)
这是 Library 的入口，也是编译器核心逻辑的唯一对外暴露点。
- **模块声明**：通过 `pub mod` 或 `mod` 组织了子模块（lexer, parser, ast, codegen 等）。
- **`compile` 函数**：这是整个流水线的总控函数。
  - **输入**：`src: &str`（ArkTS 源代码）。
  - **输出**：`Result<String, Error>`（成功返回 Rust 代码，失败返回自定义错误）。
  - **现状**：目前直接返回 `Err(Error::new("NotImplemented", ...))`，这叫“防御性编程”，先占位，后续在这个函数里串联 `lexer -> parser -> codegen`。

#### [src/main.rs](file:///Users/lyx/research/arkts2rust/src/main.rs)
这是 Binary（CLI）的入口。
- **职责**：它不包含编译逻辑，只负责 **I/O** 和 **参数解析**。
- **流程**：
  1. 获取命令行参数（简单的 `env::args()`）。
  2. 读取输入文件 (`fs::read_to_string`)。
  3. 调用 `arkts2rust::compile`。
  4. 根据结果：
     - **Ok**：写入输出文件 (`fs::write`)。
     - **Err**：打印错误信息并以非零状态码退出 (`process::exit(1)`)。

---

### 2. 基础数据结构

#### [src/span.rs](file:///Users/lyx/research/arkts2rust/src/span.rs)
用于记录代码位置，这对编译器至关重要。
- **`Span` 结构体**：通常包含 `start` (usize) 和 `end` (usize)。
- **作用**：当报错时，我们需要告诉用户是“第几个字符到第几个字符”错了。所有的 Token 和 AST 节点未来都会携带 Span。

#### [src/error.rs](file:///Users/lyx/research/arkts2rust/src/error.rs)
统一的错误处理类型。
- **`Error` 结构体**：包含 `code` (错误码/类型)、`message` (详细信息) 和 `span` (位置)。
- **实现 `Display`**：为了让 `println!("{}", err)` 能打印出人类可读的错误信息。Step0 里的实现比较简陋，后续可以优化成类似 Rust 编译器的漂亮报错。

---

### 3. 编译器流水线模块 (占位)

#### [src/lexer/](file:///Users/lyx/research/arkts2rust/src/lexer/)
词法分析层：把“字符串”变成“单词流”。
- **[mod.rs](file:///Users/lyx/research/arkts2rust/src/lexer/mod.rs)**：暴露子模块。
- **[token.rs](file:///Users/lyx/research/arkts2rust/src/lexer/token.rs)**：定义了 `Token` 和 `TokenKind`。目前是空的/占位的，Step1 我们会填入 `Ident`, `Number`, `Let`, `If` 等枚举值。
- **[lexer.rs](file:///Users/lyx/research/arkts2rust/src/lexer/lexer.rs)**：定义了 `lex` 函数签名。它的工作是吞字符，吐 Token。

#### [src/parser/](file:///Users/lyx/research/arkts2rust/src/parser/)
语法分析层：把“单词流”变成“树 (AST)”。
- **[mod.rs](file:///Users/lyx/research/arkts2rust/src/parser/mod.rs)**：暴露子模块。
- **[parser.rs](file:///Users/lyx/research/arkts2rust/src/parser/parser.rs)**：定义了 `parse` 函数签名。Step2 我们会在这里手写递归下降解析器（Recursive Descent Parser）。

#### [src/ast.rs](file:///Users/lyx/research/arkts2rust/src/ast.rs)
抽象语法树 (Abstract Syntax Tree) 定义。
- **现状**：只有空的 `Program` 和 `Stmt` 结构体。
- **未来**：这里会变成一个巨大的 Enum 集合，描述 ArkTS 的语法结构，比如 `Expr::Binary`, `Stmt::If`, `Stmt::Let` 等。

#### [src/codegen.rs](file:///Users/lyx/research/arkts2rust/src/codegen.rs)
代码生成层：把“树”变成“Rust 代码”。
- **职责**：遍历 AST，拼接字符串。
- **策略**：Step3 我们会在这里实现“ArkTS 语义 -> Rust 语义”的转换逻辑（比如 `console.log` -> `println!`）。

---

### 4. 测试与文档

#### [tests/step0_smoke.rs](file:///Users/lyx/research/arkts2rust/tests/step0_smoke.rs)
集成测试（Integration Test）。
- **位置**：`tests/` 目录下的文件会被 Cargo 视为独立的 crate 编译。
- **作用**：站在“外部使用者”的角度测试 `arkts2rust` 库。
- **内容**：目前只是调用一下 `compile`，断言它返回了预期的“未实现”错误。这证明了我们的库是可以被链接和调用的。


## 怎么跑
### 跑测试
```bash
cargo test
```

### 跑 CLI（占位版本会报错）
```bash
cargo run -- <input> -o <output>
```

预期：命令能运行，但会输出类似 `Compile failed: Error(code=NotImplemented, span=0..0)`，并以非 0 退出码结束。

## 常见错误与排查
- 找不到文件/路径错误：确认运行命令时 `<input>` 路径存在，且使用了 `-o <output>`。
- 编译不过：优先看 `cargo test` 的报错位置；Step0 只应涉及模块引用、可见性与类型定义。
- CLI 输出不清晰：确保错误打印走 `Display`（`{e}`）而不是吞掉错误。

## 可直接放进论文的方法段落（1-2 段）
本工作采用工程化的源到源编译流水线，将 ArkTS 的一个受限子集逐步转换为可编译的 Rust 代码。整体架构固定为 Lexer → Parser → AST → CodeGen → Tests，各阶段分别负责字符级切分、语法结构构建、中间表示承载与目标代码生成，并通过自动化测试保证阶段性可运行性。

在 Step0 中，我们首先建立 Cargo 项目骨架与稳定的模块边界：定义统一的 Span 与错误类型以支撑后续精确诊断，同时提供对外 `compile` API 与最简命令行入口。该阶段刻意不实现任何词法/语法/生成规则，仅保证工程结构、可编译性与测试基线成立，为后续分步骤演进提供可重复的验收起点。

---

# 附录：项目规划与规范 (Project Specification & Roadmap)

以下内容定义了本项目最终目标（Step6）的语言子集、映射规则与分阶段计划。

## A) ArkTS 子集清单（支持 / 不支持）

**总体范围（后端逻辑子集）**
- 目标覆盖：变量、表达式、基本控制流、函数、基础类型注解、console.log。
- 不追求：完整 ArkTS/TS 语义一致；只保证“尽量生成可编译 Rust”，并在文档中明确限制。

**支持（逐步递增，Step0~Step6 最终集合）**
- **词法层面**
  - 标识符、关键字、整数/字符串字面量、运算符、分隔符、注释（`//` 与 `/* */`）、空白与换行。
- **类型（非常小的子集）**
  - `number`, `string`, `boolean`, `void`（仅用于函数返回类型）；
- **声明与语句**
  - `let` / `const` 变量声明（单个标识符，带可选类型注解与可选初始化）。
  - 赋值语句：`x = expr;`
  - 表达式语句：`expr;`（包括函数调用与 `console.log(...)`）
  - `return`（可带表达式或空返回）
  - 块：`{ stmt* }`
- **表达式（Step4 完成优先级闭环）**
  - 字面量：整数、字符串、`true/false`
  - 标识符引用
  - 一元：`-expr`, `!expr`
  - 二元：`+ - * / %`, 比较 `< <= > >=`, 相等 `== !=`, 逻辑 `&& ||`
  - 括号分组 `(expr)`
  - 调用：`ident(expr, ...)` 与 `console.log(expr)`（特判映射）
- **控制流（Step5）**
  - `if (expr) stmt (else stmt)?`
  - `while (expr) stmt`
- **函数（Step6）**
  - 函数声明：`function name(params): type { ... }`
  - 参数：`name: type`（只支持按值传递；不做默认值、可选参数、rest 参数）
  - 返回类型：`void/number/string/boolean`

**明确不支持**
- UI 语法（ArkUI）、`class/interface/decorator`
- `import/export/module/namespace`
- 泛型、union/intersection、类型别名、接口合并、条件类型等复杂类型系统
- `async/await`, Promise
- 解构、展开运算符 `...`
- 数组/对象字面量、属性访问 `obj.x`、索引 `a[i]`
- `switch/try/catch/throw`
- 运算符：`=== !== ?? ?. **`、位运算等
- 自增自减 `++/--`
- 模板字符串、正则、BigInt、浮点 `1.2`

## B) 子集 EBNF（最终 Step6 版本）

```ebnf
Program         = { Stmt } EOF ;

Stmt            = VarDecl ";"
                | Assign ";"
                | ReturnStmt ";"
                | ExprStmt ";"
                | IfStmt
                | WhileStmt
                | Block
                | FuncDecl ;

VarDecl         = ("let" | "const") Ident [ TypeAnn ] [ "=" Expr ] ;

Assign          = Ident "=" Expr ;

ReturnStmt      = "return" [ Expr ] ;

ExprStmt        = Expr ;

IfStmt          = "if" "(" Expr ")" Stmt [ "else" Stmt ] ;

WhileStmt       = "while" "(" Expr ")" Stmt ;

Block           = "{" { Stmt } "}" ;

FuncDecl        = "function" Ident "(" [ Params ] ")" [ RetTypeAnn ] Block ;

Params          = Param { "," Param } ;
Param           = Ident TypeAnn ;

TypeAnn         = ":" Type ;
RetTypeAnn      = ":" Type ;

Type            = "number" | "string" | "boolean" | "void" ;

Expr            = LogicOr ;

LogicOr         = LogicAnd { "||" LogicAnd } ;
LogicAnd        = Equality { "&&" Equality } ;

Equality        = Comparison { ("==" | "!=") Comparison } ;
Comparison      = Term { ("<" | "<=" | ">" | ">=") Term } ;

Term            = Factor { ("+" | "-") Factor } ;
Factor          = Unary { ("*" | "/" | "%") Unary } ;

Unary           = ( "!" | "-" ) Unary | Call ;

Call            = Primary { "(" [ Args ] ")" } ;

Args            = Expr { "," Expr } ;

Primary         = IntLit
                | StringLit
                | BoolLit
                | Ident
                | "(" Expr ")" ;

BoolLit         = "true" | "false" ;

Ident           = IDENT_TOKEN ;
IntLit          = INT_TOKEN ;
StringLit       = STRING_TOKEN ;
```

## C) ArkTS -> Rust 映射规则表 + 限制说明

| ArkTS 子集构造 | Rust 生成 | 备注/限制 |
|---|---|---|
| `number` | `i32` | 固定策略；仅支持 32-bit 有符号整数 |
| `string` | `String` | 字面量 `"x"` -> `"x".to_string()` |
| `boolean` | `bool` | `true/false` 直映射 |
| `void` | `()` | 仅用于函数返回类型 |
| `let x = e;` | `let mut x = <e>;` | 固定策略：let 一律 mut |
| `const x = e;` | `let x = <e>;` | 固定策略：const -> let |
| `x = e;` | `x = <e>;` | 要求 `x` 已声明 |
| `console.log(e)` | `println!("{:?}", <e>)` | 固定策略；`e` 需要实现 Debug |
| `if (c) s1 else s2` | `if <c> { ... } else { ... }` | CodeGen 强制包块 |
| `while (c) s` | `while <c> { ... }` | CodeGen 强制包块 |
| `function f(a:T): R` | `fn f(a: <T>) -> <R>` | 参数按值传递 |
| `return;` | `return;` | 仅 void 函数可用 |
| 二元 `+ - * / %` | 同名运算 | 仅数值（i32） |
| `&& || !` | 同名运算 | 仅 boolean |

**限制说明**
- **类型系统极简**：只识别 `number/string/boolean/void`。
- **可变性策略保守**：`let` 一律生成 `let mut`。
- **不做完整语义等价**：溢出行为、打印格式等遵循 Rust 行为。
- **无标准库依赖**：生成代码只使用 Rust `std`。

## D) 模块职责（工程化目录）

- `src/error.rs`: 定义统一错误类型 `Error`。
- `src/span.rs`: 定义 `Span`。
- `src/lexer/{mod,token,lexer}.rs`: 词法分析，产出 Token 流。
- `src/parser/{mod,parser}.rs`: 语法分析，产出 AST。
- `src/ast.rs`: 定义 AST 结构。
- `src/codegen.rs`: AST -> Rust 源码字符串。
- `src/lib.rs`: 串联流水线，暴露 `compile` 接口。
- `src/main.rs`: CLI 入口。
- `tests/`: 黑盒测试。
- `docs/`: 步骤文档。

## E) Step0 ~ Step6 阶段计划

- **Step0：骨架** (Done)
  - 目标：项目可编译，CLI 跑通，模块占位。
- **Step1：Lexer**
  - 目标：Token 化闭环，支持所有关键字与字面量。
  - Tests：≥ 8 个。
- **Step2：Parser + AST（最小语句）**
  - 目标：VarDecl, Return, Block, console.log。
  - Tests：≥ 10 个。
- **Step3：CodeGen 闭环**
  - 目标：AST -> Rust 字符串，生成可编译代码。
  - Tests：≥ 8 个。
- **Step4：表达式优先级 + 赋值**
  - 目标：完整表达式（算术/逻辑/比较/括号），赋值语句。
  - Tests：≥ 15 个。
- **Step5：控制流**
  - 目标：If/Else, While。
  - Tests：≥ 12 个。
- **Step6：函数与类型**
  - 目标：Function Decl, Params, Return Type。
  - Tests：≥ 15 个。
