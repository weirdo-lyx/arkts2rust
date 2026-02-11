/// 源码中的一个“区间位置”。
///
/// - `start/end`：byte offset（按 UTF-8 字节计数），更适合做切片/定位。
/// - `*_line/*_col`：行列号（从 1 开始），更适合给人看的报错信息。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    /// 起始 byte offset（包含）
    pub start: usize,
    /// 结束 byte offset（不包含）
    pub end: usize,
    /// 起始行号（从 1 开始）
    pub start_line: usize,
    /// 起始列号（从 1 开始）
    pub start_col: usize,
    /// 结束行号（从 1 开始）
    pub end_line: usize,
    /// 结束列号（从 1 开始）
    pub end_col: usize,
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: 0,
            end: 0,
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    }
}

impl Span {
    /// 只设置 byte offset，行列信息使用默认值（1:1..1:1）。
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start,
            end,
            ..Self::default()
        }
    }

    /// 同时设置 byte offset 与行列信息。
    pub fn new_with_line_col(
        start: usize,
        end: usize,
        start_line: usize,
        start_col: usize,
        end_line: usize,
        end_col: usize,
    ) -> Self {
        Self {
            start,
            end,
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }
}
