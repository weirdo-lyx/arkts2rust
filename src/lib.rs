pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

// 对外公开的核心类型/函数（给 CLI、测试、以及未来可能的其它 Rust 项目使用）。
pub use error::Error;
pub use lexer::{lex, Token, TokenKind};
pub use span::Span;

/// 编译入口（Step0/Step1 仍是占位实现）。
///
/// 注意：按照“分步交付”原则，Step1 只实现 Lexer，不允许让 compile 进入 Parser/CodeGen。
pub fn compile(_src: &str) -> Result<String, Error> {
    Err(Error::new("NotImplemented", Span::default()))
}
