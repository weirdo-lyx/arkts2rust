pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

pub use error::Error;
pub use span::Span;

pub fn compile(_src: &str) -> Result<String, Error> {
    Err(Error::new("NotImplemented", Span::default()))
}
