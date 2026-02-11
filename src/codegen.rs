use crate::ast::Program;
use crate::error::Error;
use crate::span::Span;

pub fn generate(_program: &Program) -> Result<String, Error> {
    Err(Error::new("NotImplemented", Span::default()))
}
