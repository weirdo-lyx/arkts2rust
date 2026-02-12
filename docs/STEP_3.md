# STEP 3：代码生成（CodeGen）——把 AST 生成 Rust 源码

本 Step 的目标：把 Step2 解析出的 AST 转换成一段 Rust 源码字符串，从而形成最小闭环：

```text
ArkTS 源码 -> Lexer(Token) -> Parser(AST) -> CodeGen(Rust 源码字符串)
```

## 1. 什么是 CodeGen

CodeGen（代码生成）就是把“结构化的 AST”重新转换成目标语言的源代码文本。

注意：本项目是源到源编译器，所以 CodeGen 的输出是 **Rust 源码字符串**，而不是字节码/机器码。

## 2. 本 Step 仅支持 Step2 的最小 AST

语法范围不扩展，仍只支持：
- `let/const` 声明：`let x = literal;` / `const x = literal;`
- 表达式语句：`console.log(literal);`
- `literal`：number/string/boolean
- 每条语句必须以 `;` 结束

## 3. 映射规则（ArkTS -> Rust）

CodeGen 规则如下（实现见 `src/codegen.rs`）：

### 3.1 程序结构
- 输出完整 Rust 程序：

```rust
fn main() {
    // ...
}
```

### 3.2 变量声明
- `let` -> `let mut`
- `const` -> `let`

示例：

```text
let x = 1;
const s = "hi";
```

生成：

```rust
fn main() {
    let mut x = 1i32;
    let s = String::from("hi");
}
```

### 3.3 字面量类型
- number -> `i32`（生成时用 `1i32` 这种字面量后缀固定类型）
- string -> `String`（统一使用 `String::from("...")`）
- boolean -> `bool`

### 3.4 console.log
- `console.log(e)` -> `println!("{:?}", e)`

例如：

```text
console.log(true);
```

生成：

```rust
fn main() {
    println!("{:?}", true);
}
```

## 4. 如何使用 CLI 生成 output.rs

CLI 入口是 `src/main.rs`，用法：

```bash
cargo run -- <input.ets> [-o <output.rs>]
```

- 如果不传 `-o`，默认输出为 `output.rs`

示例：

```bash
cargo run -- input.ets -o output.rs
```

## 5. 如何验证生成的 Rust 能编译

手动验证步骤：
1. 运行 CLI 生成 Rust 文件：

```bash
cargo run -- input.ets -o output.rs
```

2. 用 rustc 编译：

```bash
rustc output.rs
```

如果编译成功，会得到可执行文件（在 macOS/Linux 下一般叫 `output`）。

## 6. 测试（黄金用例）

本 Step 使用“黄金测试”（golden tests）确保生成的 Rust 代码稳定一致：
- `tests/golden_tests.rs` 内对比 `compile(src)` 的输出字符串与期望字符串
- 至少包含 6 个用例

运行：

```bash
cargo test
```

## 7. 论文段落（可直接引用/改写）

在源到源编译器中，代码生成阶段负责将语法分析得到的抽象语法树（AST）映射为目标语言的等价源代码表示。本文在 ArkTS 子集上实现了面向 Rust 的代码生成器，采用基于节点类型的递归遍历策略，将变量声明、字面量与特定调用表达式映射为 Rust 的 `let/let mut`、`i32/String/bool` 以及 `println!` 等结构。通过黄金测试对生成结果进行稳定性验证，并使用手动编译步骤确认生成的 Rust 程序能够通过编译，从而完成从输入源代码到可编译目标代码的最小闭环。

