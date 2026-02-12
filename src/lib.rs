pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

/// crate 的模块导出。
///
/// 小白视角可以这样理解：
/// - `src/lib.rs`：库（library）的“门面”，把内部模块组织好并对外暴露 API。
/// - `src/main.rs`：可执行程序（binary）的入口，通常只负责 I/O（读文件、写文件、打印错误）。
///
/// 这样拆分的好处：
/// - 测试更方便：tests/ 更像“外部用户”，只调用 lib 暴露的函数。
/// - 复用更容易：未来其它 Rust 项目也能直接依赖这个库。
pub use ast::{
    Callee, CallExpr, Expr, FuncDecl, Literal, Param, Program, Stmt, TypeAnn, VarDecl,
};
pub use error::Error;
pub use lexer::{lex, Token, TokenKind};
pub use parser::parse as parse_tokens;
pub use span::Span;

/// 辅助函数：直接从源代码解析出 Program AST。
///
/// 这对测试 Parser 很方便：不需要手动先调用 lex()。
pub fn parse_program(src: &str) -> Result<Program, Error> {
    let tokens = lex(src)?;
    parse_tokens(&tokens)
}

/// 编译入口：把 ArkTS 子集源码编译成 Rust 源码字符串。
///
/// 目前 Step3 的流水线是：
/// 1. Lexer：`src` -> `Vec<Token>`
/// 2. Parser：`Vec<Token>` -> `Program` AST
/// 3. CodeGen：`Program` -> Rust 源码字符串
///
/// 注意：这一步的“compile”只生成 Rust 源码，不会自动调用 rustc 去编译。
pub fn compile(src: &str) -> Result<String, Error> {
    let tokens = lex(src)?;
    let program = parse_tokens(&tokens)?;
    codegen::generate(&program)
}
