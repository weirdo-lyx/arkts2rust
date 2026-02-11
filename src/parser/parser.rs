use crate::ast::Program;
use crate::error::Error;
use crate::lexer::token::Token;
use crate::span::Span;

pub fn parse(_tokens: &[Token]) -> Result<Program, Error> {
    Err(Error::new("NotImplemented", Span::default()))
}
