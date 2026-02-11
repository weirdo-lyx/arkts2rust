use crate::span::Span;
use std::fmt;

/// 编译器统一错误类型。
///
/// 设计要点：
/// - `code`：机器可读的错误码（便于测试断言、分类统计）。
/// - `span`：错误发生的位置（byte offset + line/col），便于定位。
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    pub code: String,
    pub span: Span,
}

impl Error {
    /// 创建一个错误。`code` 通常使用短字符串（例如 `UnexpectedChar`）。
    pub fn new(code: impl Into<String>, span: Span) -> Self {
        Self {
            code: code.into(),
            span,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Display 用于给人看的错误信息，CLI 会直接打印它。
        write!(
            f,
            "Error(code={}, span={}..{}, loc={}:{}..{}:{})",
            self.code,
            self.span.start,
            self.span.end,
            self.span.start_line,
            self.span.start_col,
            self.span.end_line,
            self.span.end_col
        )
    }
}

impl std::error::Error for Error {}
