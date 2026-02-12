pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

// 对外公开的核心类型/函数（给 CLI、测试、以及未来可能的其它 Rust 项目使用）。
pub use ast::{Callee, CallExpr, Expr, Literal, Program, Stmt, VarDecl};
pub use error::Error;
pub use lexer::{lex, Token, TokenKind};
pub use parser::parse as parse_tokens;
pub use span::Span;

pub fn parse_program(src: &str) -> Result<Program, Error> {
    let tokens = lex(src)?;
    parse_tokens(&tokens)
}

pub fn compile(src: &str) -> Result<String, Error> {
    let tokens = lex(src)?;
    let program = parse_tokens(&tokens)?;
    codegen::generate(&program)
}
