# Step 1：Lexer / Tokenizer（词法分析）

## 本 Step 新增了什么
- 定义 Token 类型：关键字、标识符、数字、字符串、符号、运算符。
- 实现词法分析器 `lex(src) -> Result<Vec<Token>, Error>`：
  - 跳过空白与注释（支持 `//` 单行注释；额外支持 `/* */` 块注释）。
  - 每个 Token 带 `Span`（byte offset + line/col）。
  - 非法字符、未闭合字符串会报错并指出位置。
- 新增集成测试 `tests/lexer_tests.rs`（≥ 12 个），覆盖关键路径与错误用例。

## 本 Step 允许修改的文件白名单
- `src/lexer/token.rs`
- `src/lexer/lexer.rs`
- `src/lexer/mod.rs`（仅导出）
- `src/span.rs`
- `src/error.rs`
- `src/lib.rs`（可暴露 lex 接口供测试，但 compile 不能进入 parser/codegen）
- `tests/lexer_tests.rs`
- `docs/STEP_1.md`

## 小白解释：Lexer 到底在做什么
Lexer（词法分析）做的事情很“朴素”：把源代码这串字符切成一个个 Token。

你可以把 Token 理解成“语法积木”：
- `let` 是一个关键字 Token
- `x` 是一个标识符 Token
- `123` 是一个数字 Token
- `==` 是一个运算符 Token
- `(` `)` `{` `}` `,` `;` 是符号 Token

Parser（语法分析）在 Step2 才会登场：它会在 Token 序列上再“搭结构”，构建 AST。Step1 只负责把字符切好，并尽量提供准确的位置信息以便报错。

## 位置（Span）为什么要包含 byte offset + line/col
- **byte offset**：适合做切片/定位（Rust 的字符串切片通常以 byte 为单位）。
- **line/col**：对人友好，报错时能说“第几行第几列”，用户更容易定位。

本项目的 `Span` 同时保存两类信息，并让 `Error` 打印时包含 `loc=line:col..line:col`。

## 调试方法（遇到 Lexer 错误怎么查）
- 先看错误码 `Error.code`：
  - `UnexpectedChar`：遇到了不在子集里的字符（比如 `@`）。
  - `UnterminatedString`：字符串没有闭合（比如只有开头 `"` 没有结尾 `"`）。
  - `UnterminatedBlockComment`：块注释没闭合（`/*` 没有对应 `*/`）。
- 再看 `Error.span.start_line/start_col`：
  - 把它当作“报错的起点位置”。
  - 用编辑器跳到对应行列，检查附近字符是否符合子集规则。

## 怎么跑
```bash
cargo test
```

## 可直接放进论文的方法段落（1-2 段）
本研究采用分阶段构建的源到源编译策略。首先在词法分析阶段（Lexer）中，将输入的 ArkTS 子集源代码视为字符流，通过顺序扫描与最长匹配规则，将其切分为一系列 Token（关键字、标识符、字面量、运算符与分隔符），并在切分过程中忽略空白与注释，从而为后续语法分析提供结构化输入。

为提升可调试性与可复现实验结果，本文在 Token 与错误诊断中引入 Span 机制，同时记录 byte offset 与行列号（line/column）。当检测到非法字符或未闭合字符串等词法错误时，编译器返回携带 Span 的结构化错误信息，使得错误定位能够稳定映射回源代码位置，便于自动化测试与人工排查。
