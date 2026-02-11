use crate::span::Span;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: String,
    pub span: Span,
}

impl Error {
    pub fn new(code: impl Into<String>, span: Span) -> Self {
        Self {
            code: code.into(),
            span,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Error(code={}, span={}..{})",
            self.code, self.span.start, self.span.end
        )
    }
}

impl std::error::Error for Error {}
