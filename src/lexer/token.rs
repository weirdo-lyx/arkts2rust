use crate::span::Span;

/// 一个 Token = 词法分析后的最小“语法积木”。
///
/// 例子：`let x = 1;`
/// 会被切成：KwLet, Ident("x"), Eq, Number(1), Semicolon
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Token {
    /// Token 的类别（关键字/标识符/字面量/运算符/分隔符等）
    pub kind: TokenKind,
    /// Token 在源代码中的位置
    pub span: Span,
}

/// Token 的种类枚举。
///
/// 注意：Step1 只负责“把字符切成 Token”，不负责语法结构（那是 Step2 Parser 的工作）。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TokenKind {
    // ---------- 关键字 ----------
    KwLet,
    KwConst,
    KwFunction,
    KwIf,
    KwElse,
    KwWhile,
    KwReturn,
    KwTrue,
    KwFalse,

    // ---------- 语义性 Token（携带值） ----------
    /// 标识符：例如 `abc`、`x1`、`_tmp`
    Ident(String),
    /// 整数字面量（ArkTS number 子集在后续会映射为 Rust i32，所以这里直接存 i32）
    Number(i32),
    /// 字符串字面量（支持少量转义）
    String(String),

    // ---------- 分隔符 / 符号 ----------
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    Dot,
    Semicolon,

    // ---------- 运算符 ----------
    Plus,
    Minus,
    Star,
    Slash,
    Percent,

    EqEq,
    NotEq,
    LtEq,
    GtEq,
    Lt,
    Gt,

    AndAnd,
    OrOr,
    Not,
    Eq,
}
