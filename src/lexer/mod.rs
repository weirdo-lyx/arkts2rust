/// Lexer 模块：负责把源代码字符串切成 Token 序列。
pub mod lexer;
pub mod token;

/// 对外导出：`lex(src)` 入口函数。
pub use lexer::lex;
/// 对外导出：Token 数据结构。
pub use token::{Token, TokenKind};
