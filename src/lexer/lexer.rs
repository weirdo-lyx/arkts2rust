use crate::error::Error;
use crate::lexer::token::{Token, TokenKind};
use crate::span::Span;

pub fn lex(_src: &str) -> Result<Vec<Token>, Error> {
    Err(Error::new("NotImplemented", Span::default()))
}

pub fn eof_token() -> Token {
    Token {
        kind: TokenKind::Eof,
        span: Span::default(),
    }
}
