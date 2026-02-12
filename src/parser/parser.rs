use crate::ast::{
    AssignStmt, BinaryExpr, BinaryOp, Callee, CallExpr, Expr, Literal, Program, Stmt, UnaryExpr,
    UnaryOp, VarDecl,
};
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
    /// - `Ident = Expr ;` -> Assign
    /// - 其它 -> parse_expr + 分号
    ///
    /// 说明：Step2 的语法要求“每条语句都必须以分号结尾”。
    /// 因为 `;` 不属于表达式本身，所以这里统一在语句层做检查。
    fn parse_stmt(&mut self) -> Result<Stmt, Error> {
        match self.peek_kind() {
            Some(TokenKind::KwLet) => self.parse_var_decl(false),
            Some(TokenKind::KwConst) => self.parse_var_decl(true),
            Some(TokenKind::Ident(_)) if matches!(self.peek_kind_n(1), Some(TokenKind::Eq)) => {
                let name = self.expect_ident()?;
                self.expect_simple(TokenKind::Eq)?;
                let value = self.parse_expr_bp(0)?;
                self.expect_semicolon()?;
                Ok(Stmt::Assign(AssignStmt { name, value }))
            }
            _ => {
                let expr = self.parse_expr_bp(0)?;
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

    /// 解析表达式（Pratt Parser / 运算符优先级解析）。
    ///
    /// `min_bp` 是当前允许的“最小绑定强度”（binding power）。
    /// - 数值越大，绑定越紧（优先级越高）。
    /// - 在 while 循环里不断吃掉可以绑定到左侧的运算符，从而构建正确的 AST 结构。
    ///
    /// Step4 支持的优先级（从低到高，简化版）：
    /// 1) `||`
    /// 2) `&&`
    /// 3) `==` `!=`
    /// 4) `<` `<=` `>` `>=`
    /// 5) `+` `-`
    /// 6) `*` `/` `%`
    /// 7) 前缀 `!` `-`
    /// 8) 调用 `f(...)`（后缀，绑定最紧）
    fn parse_expr_bp(&mut self, min_bp: u8) -> Result<Expr, Error> {
        let mut lhs = self.parse_prefix()?;

        loop {
            // ---------- 处理函数调用：ident(expr, expr, ...) ----------
            if matches!(self.peek_kind(), Some(TokenKind::LParen)) {
                let (l_bp, _r_bp) = (15u8, 16u8);
                if l_bp < min_bp {
                    break;
                }

                let lparen_span = self.peek_span().unwrap_or_default();

                match lhs {
                    Expr::Ident(name) => {
                        let args = self.parse_call_args()?;
                        lhs = Expr::Call(CallExpr {
                            callee: Callee::Ident(name),
                            args,
                        });
                        continue;
                    }
                    Expr::Call(_) => {
                        return Err(Error::new("UnknownStructure", lparen_span));
                    }
                    _ => {
                        return Err(Error::new("UnknownStructure", lparen_span));
                    }
                }
            }

            // ---------- 处理二元运算 ----------
            let (l_bp, r_bp, op) = match self.peek_kind().and_then(|k| infix_bp(k)) {
                Some(x) => x,
                None => break,
            };

            if l_bp < min_bp {
                break;
            }

            let _op_tok = self.bump();
            let rhs = self.parse_expr_bp(r_bp)?;
            lhs = Expr::Binary(BinaryExpr {
                op,
                left: Box::new(lhs),
                right: Box::new(rhs),
            });
        }

        Ok(lhs)
    }

    /// 解析前缀表达式（primary / unary）。
    fn parse_prefix(&mut self) -> Result<Expr, Error> {
        match self.peek_kind() {
            Some(TokenKind::Not) => {
                let _ = self.bump();
                let rhs = self.parse_expr_bp(13)?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Not,
                    expr: Box::new(rhs),
                }))
            }
            Some(TokenKind::Minus) => {
                let _ = self.bump();
                let rhs = self.parse_expr_bp(13)?;
                Ok(Expr::Unary(UnaryExpr {
                    op: UnaryOp::Neg,
                    expr: Box::new(rhs),
                }))
            }
            _ => self.parse_primary(),
        }
    }

    /// 解析“最基础”的表达式单元（primary）。
    ///
    /// 支持：
    /// - literal：number/string/boolean
    /// - ident：标识符引用
    /// - 括号：`(expr)`
    /// - console.log(literal)：为了兼容 Step2/Step3（保持 console.log 参数仍是 literal）
    fn parse_primary(&mut self) -> Result<Expr, Error> {
        match self.peek_kind() {
            Some(TokenKind::Number(_))
            | Some(TokenKind::String(_))
            | Some(TokenKind::KwTrue)
            | Some(TokenKind::KwFalse) => Ok(Expr::Literal(self.parse_literal()?)),
            Some(TokenKind::Ident(s)) if s == "console.log" => self.parse_console_log_call(),
            Some(TokenKind::Ident(s)) if s == "console" => {
                if matches!(self.peek_kind_n(1), Some(TokenKind::Dot))
                    && matches!(self.peek_kind_n(2), Some(TokenKind::Ident(_)))
                {
                    self.parse_console_log_call()
                } else {
                    Ok(Expr::Ident(self.expect_ident()?))
                }
            }
            Some(TokenKind::Ident(_)) => Ok(Expr::Ident(self.expect_ident()?)),
            Some(TokenKind::LParen) => {
                let _ = self.bump();
                let inner = self.parse_expr_bp(0)?;
                self.expect_rparen()?;
                Ok(Expr::Group(Box::new(inner)))
            }
            Some(_) => Err(self.err_here("ExpectedExpr")),
            None => Err(self.err_eof("ExpectedExpr")),
        }
    }

    /// 解析 console.log(literal) 调用（兼容 Step2/Step3）。
    ///
    /// 注意：为了不破坏原来的 Step2 测试，这里仍然严格要求参数是 literal。
    fn parse_console_log_call(&mut self) -> Result<Expr, Error> {
        let start_span = self.peek_span().unwrap_or_default();

        let callee = match self.peek_kind() {
            Some(TokenKind::Ident(s)) if s == "console.log" => {
                let _ = self.bump();
                Callee::ConsoleLog
            }
            Some(TokenKind::Ident(s)) if s == "console" => {
                let _ = self.bump();
                self.expect_dot()?;
                let ident = self.expect_ident()?;
                if ident != "log" {
                    return Err(self.err_span("UnknownStructure", start_span));
                }
                Callee::ConsoleLog
            }
            _ => return Err(self.err_here("UnknownStructure")),
        };

        self.expect_simple(TokenKind::LParen)?;
        let arg = Expr::Literal(self.parse_literal()?);
        let args = vec![arg];
        self.expect_rparen()?;
        Ok(Expr::Call(CallExpr { callee, args }))
    }

    /// 解析函数调用参数列表（用于 ident(expr, expr, ...)）。
    ///
    /// 进入本函数时，当前 token 必须是 `(`。
    fn parse_call_args(&mut self) -> Result<Vec<Expr>, Error> {
        self.expect_simple(TokenKind::LParen)?;

        let mut args = Vec::new();
        if matches!(self.peek_kind(), Some(TokenKind::RParen)) {
            let _ = self.bump();
            return Ok(args);
        }

        loop {
            let expr = self.parse_expr_bp(0)?;
            args.push(expr);

            match self.peek_kind() {
                Some(TokenKind::Comma) => {
                    let _ = self.bump();
                }
                Some(TokenKind::RParen) => {
                    let _ = self.bump();
                    break;
                }
                Some(_) => return Err(self.err_here("MissingRParen")),
                None => return Err(self.err_eof("MissingRParen")),
            }
        }

        Ok(args)
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

    /// 向前偷看第 n 个 token 的 kind（不前进）。
    ///
    /// 例：`peek_kind_n(1)` 表示看“下一个 token”，`peek_kind_n(2)` 表示看“下下个 token”。
    fn peek_kind_n(&self, n: usize) -> Option<&TokenKind> {
        self.tokens.get(self.i + n).map(|t| &t.kind)
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

fn infix_bp(kind: &TokenKind) -> Option<(u8, u8, BinaryOp)> {
    // 这里返回 (left_bp, right_bp, op)：
    // - left_bp 越大，表示该运算符越“紧密地绑定”左侧
    // - right_bp 越大，表示该运算符越“紧密地绑定”右侧
    //
    // 左结合实现技巧：
    // - 对左结合运算符（本 Step 的所有二元运算符都是左结合），使用 (p, p+1)
    //   能确保 `1-2-3` 解析为 `(1-2)-3`，而不是 `1-(2-3)`。
    match kind {
        TokenKind::OrOr => Some((1, 2, BinaryOp::OrOr)),
        TokenKind::AndAnd => Some((3, 4, BinaryOp::AndAnd)),
        TokenKind::EqEq => Some((5, 6, BinaryOp::EqEq)),
        TokenKind::NotEq => Some((5, 6, BinaryOp::NotEq)),
        TokenKind::Lt => Some((7, 8, BinaryOp::Lt)),
        TokenKind::LtEq => Some((7, 8, BinaryOp::LtEq)),
        TokenKind::Gt => Some((7, 8, BinaryOp::Gt)),
        TokenKind::GtEq => Some((7, 8, BinaryOp::GtEq)),
        TokenKind::Plus => Some((9, 10, BinaryOp::Add)),
        TokenKind::Minus => Some((9, 10, BinaryOp::Sub)),
        TokenKind::Star => Some((11, 12, BinaryOp::Mul)),
        TokenKind::Slash => Some((11, 12, BinaryOp::Div)),
        TokenKind::Percent => Some((11, 12, BinaryOp::Mod)),
        _ => None,
    }
}
