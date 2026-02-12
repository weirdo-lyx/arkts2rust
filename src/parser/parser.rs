use crate::ast::{Callee, CallExpr, Expr, Literal, Program, Stmt, VarDecl};
use crate::error::Error;
use crate::lexer::token::Token;
use crate::lexer::token::TokenKind;
use crate::span::Span;

/// 解析器入口：将 Token 列表解析为 Program AST。
pub fn parse(tokens: &[Token]) -> Result<Program, Error> {
    Parser::new(tokens).parse_program()
}

/// 递归下降解析器结构体。
///
/// 小白理解版：
/// - Parser 就像一个“指针”，在 Token 列表上从左到右走。
/// - `i` 表示当前看到了第几个 token（类似光标）。
/// - `peek_*` 表示“偷看一下”，不移动光标。
/// - `bump()` 表示“吃掉一个 token”，光标向右移动一格。
struct Parser<'a> {
    tokens: &'a [Token], // Token 流
    i: usize,            // 当前扫描位置
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, i: 0 }
    }

    /// 解析整个程序（Program = { Stmt }）
    ///
    /// 规则：一直解析语句直到 token 用完（EOF）。
    fn parse_program(&mut self) -> Result<Program, Error> {
        let mut stmts = Vec::new();
        while !self.is_eof() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(Program { stmts })
    }

    /// 解析单条语句（Stmt）
    /// - `let/const` -> parse_var_decl
    /// - 其它 -> parse_expr + 分号
    ///
    /// 说明：Step2 的语法要求“每条语句都必须以分号结尾”。
    /// 因为 `;` 不属于表达式本身，所以这里统一在语句层做检查。
    fn parse_stmt(&mut self) -> Result<Stmt, Error> {
        match self.peek_kind() {
            Some(TokenKind::KwLet) => self.parse_var_decl(false),
            Some(TokenKind::KwConst) => self.parse_var_decl(true),
            _ => {
                let expr = self.parse_expr()?;
                self.expect_semicolon()?;
                Ok(Stmt::ExprStmt(expr))
            }
        }
    }

    /// 解析变量声明（let x = ...;）
    ///
    /// 产生式（简化写法）：
    /// - `("let" | "const") Ident "=" Literal ";"`（分号在 parse_stmt 里检查，这里也会检查一次以更直观）
    fn parse_var_decl(&mut self, is_const: bool) -> Result<Stmt, Error> {
        if is_const {
            self.expect_simple(TokenKind::KwConst)?;
        } else {
            self.expect_simple(TokenKind::KwLet)?;
        }

        let name = self.expect_ident()?; // 变量名
        self.expect_simple(TokenKind::Eq)?; // 等号
        let lit = self.parse_literal()?; // 初始值
        self.expect_semicolon()?; // 分号
        Ok(Stmt::VarDecl(VarDecl {
            is_const,
            name,
            init: lit,
        }))
    }

    /// 解析表达式（Expr）
    /// - 数字/字符串/true/false -> Literal
    /// - 标识符 -> CallExpr (目前只支持 console.log)
    ///
    /// Step2 不支持复杂表达式（例如 1 + 2 * 3），所以这里不处理优先级。
    fn parse_expr(&mut self) -> Result<Expr, Error> {
        match self.peek_kind() {
            Some(TokenKind::Number(_))
            | Some(TokenKind::String(_))
            | Some(TokenKind::KwTrue)
            | Some(TokenKind::KwFalse) => Ok(Expr::Literal(self.parse_literal()?)),
            Some(TokenKind::Ident(_)) => self.parse_call_expr(),
            Some(_) => Err(self.err_here("UnexpectedToken")),
            None => Err(self.err_eof("UnexpectedEof")),
        }
    }

    /// 解析函数调用（目前特指 console.log(...)）
    ///
    /// 支持的两种 token 形式：
    /// - `Ident("console") Dot Ident("log") ...`
    /// - （备用兼容）`Ident("console.log") ...`
    fn parse_call_expr(&mut self) -> Result<Expr, Error> {
        let start_span = self.peek_span().unwrap_or_default();

        // 识别 `console.log`：
        // 1. 如果 lexer 直接识别了 "console.log"（如果是标识符允许点号的情况，但在当前 token 定义中点号是独立的）
        // 2. 实际上 Token 序列是 [Ident(console), Dot, Ident(log)]
        let callee = match self.peek_kind() {
            // Case A: 如果直接匹配到 "console.log" 字符串（理论上 Step1 lexer 不会产生含点的 Ident，除非改了）
            // 但我们的 lexer 把 '.' 视为独立符号，所以这里需要多步匹配。
            Some(TokenKind::Ident(s)) if s == "console.log" => {
                let _ = self.bump();
                Callee::ConsoleLog
            }
            // Case B: 匹配 `console` -> `.` -> `log`
            Some(TokenKind::Ident(s)) if s == "console" => {
                let _ = self.bump();
                self.expect_dot()?;
                let ident = self.expect_ident()?;
                if ident != "log" {
                    return Err(self.err_span("UnknownStructure", start_span));
                }
                Callee::ConsoleLog
            }
            Some(TokenKind::Ident(_)) => return Err(self.err_here("UnknownStructure")),
            _ => return Err(self.err_here("UnexpectedToken")),
        };

        self.expect_simple(TokenKind::LParen)?;
        // Step2 要求参数是 literal，因此这里直接 parse_literal()
        let arg = Expr::Literal(self.parse_literal()?);
        let args = vec![arg];
        self.expect_rparen()?;
        Ok(Expr::Call(CallExpr { callee, args }))
    }

    /// 解析字面量（Literal）
    ///
    /// 如果当前 token 不是字面量，会返回 `ExpectedLiteral` 错误。
    fn parse_literal(&mut self) -> Result<Literal, Error> {
        match self.peek_kind() {
            Some(TokenKind::Number(n)) => {
                let n = *n;
                let _ = self.bump();
                Ok(Literal::Number(n))
            }
            Some(TokenKind::String(s)) => {
                let s = s.clone();
                let _ = self.bump();
                Ok(Literal::String(s))
            }
            Some(TokenKind::KwTrue) => {
                let _ = self.bump();
                Ok(Literal::Bool(true))
            }
            Some(TokenKind::KwFalse) => {
                let _ = self.bump();
                Ok(Literal::Bool(false))
            }
            Some(_) => Err(self.err_here("ExpectedLiteral")),
            None => Err(self.err_eof("ExpectedLiteral")),
        }
    }

    /// 期望下一个 token 是标识符（Ident），并返回其字符串内容。
    fn expect_ident(&mut self) -> Result<String, Error> {
        match self.peek_kind() {
            Some(TokenKind::Ident(s)) => {
                let s = s.clone();
                let _ = self.bump();
                Ok(s)
            }
            Some(_) => Err(self.err_here("ExpectedIdentifier")),
            None => Err(self.err_eof("ExpectedIdentifier")),
        }
    }

    /// 期望下一个 token 是分号 `;`，否则报 `MissingSemicolon`。
    fn expect_semicolon(&mut self) -> Result<(), Error> {
        match self.peek_kind() {
            Some(TokenKind::Semicolon) => {
                let _ = self.bump();
                Ok(())
            }
            Some(_) => Err(self.err_here("MissingSemicolon")),
            None => Err(self.err_eof("MissingSemicolon")),
        }
    }

    /// 期望下一个 token 是右括号 `)`，否则报 `MissingRParen`。
    fn expect_rparen(&mut self) -> Result<(), Error> {
        match self.peek_kind() {
            Some(TokenKind::RParen) => {
                let _ = self.bump();
                Ok(())
            }
            Some(_) => Err(self.err_here("MissingRParen")),
            None => Err(self.err_eof("MissingRParen")),
        }
    }

    /// 期望下一个 token 是点号 `.`，用于识别 `console.log` 里的 `.`。
    fn expect_dot(&mut self) -> Result<(), Error> {
        match self.peek_kind() {
            Some(TokenKind::Dot) => {
                let _ = self.bump();
                Ok(())
            }
            Some(_) => Err(self.err_here("ExpectedDot")),
            None => Err(self.err_eof("ExpectedDot")),
        }
    }

    /// 期望下一个 token 是某些“固定符号/关键字”。
    ///
    /// 这里没有做一个通用的 token 比较函数，而是仅覆盖 Step2 会用到的那几个 token，
    /// 让实现保持最小且更直观。
    fn expect_simple(&mut self, kind: TokenKind) -> Result<(), Error> {
        match (self.peek_kind(), &kind) {
            (Some(TokenKind::KwLet), TokenKind::KwLet)
            | (Some(TokenKind::KwConst), TokenKind::KwConst)
            | (Some(TokenKind::LParen), TokenKind::LParen)
            | (Some(TokenKind::RParen), TokenKind::RParen)
            | (Some(TokenKind::Eq), TokenKind::Eq) => {
                let _ = self.bump();
                Ok(())
            }
            (Some(_), _) => Err(self.err_here("UnexpectedToken")),
            (None, _) => Err(self.err_eof("UnexpectedEof")),
        }
    }

    /// 偷看当前 token 的 kind（不前进）。
    fn peek_kind(&self) -> Option<&TokenKind> {
        self.tokens.get(self.i).map(|t| &t.kind)
    }

    /// 偷看当前 token 的 span（不前进）。
    fn peek_span(&self) -> Option<Span> {
        self.tokens.get(self.i).map(|t| t.span)
    }

    /// 吃掉一个 token，并让光标右移一格。
    fn bump(&mut self) -> Option<&'a Token> {
        let tok = self.tokens.get(self.i);
        if tok.is_some() {
            self.i += 1;
        }
        tok
    }

    fn is_eof(&self) -> bool {
        self.i >= self.tokens.len()
    }

    /// 构造一个错误：定位到“当前 token”的 span。
    ///
    /// 如果已经没有 token（EOF），就退化为使用最后一个 token 的 span（见 eof_span）。
    fn err_here(&self, code: &'static str) -> Error {
        Error::new(code, self.peek_span().unwrap_or_else(|| self.eof_span()))
    }

    /// 构造一个错误：定位到 EOF（使用最后一个 token 的 span）。
    fn err_eof(&self, code: &'static str) -> Error {
        Error::new(code, self.eof_span())
    }

    /// 构造一个错误：定位到指定 span。
    fn err_span(&self, code: &'static str, span: Span) -> Error {
        Error::new(code, span)
    }

    /// 计算一个“EOF 时的 span”。
    ///
    /// - 如果 tokens 非空：使用最后一个 token 的 span（至少能落在文件末尾附近）
    /// - 如果 tokens 为空：使用默认 span（1:1..1:1）
    fn eof_span(&self) -> Span {
        self.tokens.last().map(|t| t.span).unwrap_or_default()
    }
}
