use crate::error::Error;
use crate::lexer::token::{Token, TokenKind};
use crate::span::Span;

/// 词法分析入口：把源代码切成一串 Token。
///
/// Step1 目标：
/// - 支持关键字/标识符/数字/字符串/运算符/符号
/// - 跳过空白与注释
/// - 出错时返回携带 Span 的 Error（包含 line/col）
pub fn lex(src: &str) -> Result<Vec<Token>, Error> {
    Lexer::new(src).lex_all()
}

/// 词法分析器的内部状态（扫描指针）。
///
/// 这里用 `byte_pos` 保存当前位置的 byte offset（UTF-8）。
/// 同时维护 `line/col` 方便报错定位。
struct Lexer<'a> {
    src: &'a str,
    byte_pos: usize,
    line: usize,
    col: usize,
}

impl<'a> Lexer<'a> {
    fn new(src: &'a str) -> Self {
        Self {
            src,
            byte_pos: 0,
            line: 1,
            col: 1,
        }
    }

    /// 扫描整个输入，直到 EOF。
    fn lex_all(mut self) -> Result<Vec<Token>, Error> {
        let mut tokens = Vec::new();

        while !self.is_eof() {
            // 先跳过空白和注释，保证下一个字符是“有意义的 Token 起点”
            self.skip_ws_and_comments()?;
            if self.is_eof() {
                break;
            }

            // 记录 token 起点位置（byte offset + line/col）
            let start_pos = self.mark();
            let ch = self.peek_char().ok_or_else(|| self.err_at("UnexpectedEof", start_pos))?;

            // 根据当前字符决定要识别哪一种 token
            let kind = match ch {
                '(' => {
                    self.bump_char();
                    TokenKind::LParen
                }
                ')' => {
                    self.bump_char();
                    TokenKind::RParen
                }
                '{' => {
                    self.bump_char();
                    TokenKind::LBrace
                }
                '}' => {
                    self.bump_char();
                    TokenKind::RBrace
                }
                ',' => {
                    self.bump_char();
                    TokenKind::Comma
                }
                '.' => {
                    self.bump_char();
                    TokenKind::Dot
                }
                ';' => {
                    self.bump_char();
                    TokenKind::Semicolon
                }
                '+' => {
                    self.bump_char();
                    TokenKind::Plus
                }
                '-' => {
                    self.bump_char();
                    TokenKind::Minus
                }
                '*' => {
                    self.bump_char();
                    TokenKind::Star
                }
                '/' => {
                    self.bump_char();
                    TokenKind::Slash
                }
                '%' => {
                    self.bump_char();
                    TokenKind::Percent
                }
                '=' => {
                    self.bump_char();
                    // 匹配 `==` 或 `=`
                    if self.try_bump('=') {
                        TokenKind::EqEq
                    } else {
                        TokenKind::Eq
                    }
                }
                '!' => {
                    self.bump_char();
                    // 匹配 `!=` 或 `!`
                    if self.try_bump('=') {
                        TokenKind::NotEq
                    } else {
                        TokenKind::Not
                    }
                }
                '<' => {
                    self.bump_char();
                    // 匹配 `<=` 或 `<`
                    if self.try_bump('=') {
                        TokenKind::LtEq
                    } else {
                        TokenKind::Lt
                    }
                }
                '>' => {
                    self.bump_char();
                    // 匹配 `>=` 或 `>`
                    if self.try_bump('=') {
                        TokenKind::GtEq
                    } else {
                        TokenKind::Gt
                    }
                }
                '&' => {
                    self.bump_char();
                    // 只支持 `&&`，单独的 `&` 在子集中是非法字符
                    if self.try_bump('&') {
                        TokenKind::AndAnd
                    } else {
                        return Err(self.err_at("UnexpectedChar", start_pos));
                    }
                }
                '|' => {
                    self.bump_char();
                    // 只支持 `||`，单独的 `|` 在子集中是非法字符
                    if self.try_bump('|') {
                        TokenKind::OrOr
                    } else {
                        return Err(self.err_at("UnexpectedChar", start_pos));
                    }
                }
                '"' => self.lex_string()?,
                c if c.is_ascii_digit() => self.lex_number()?,
                c if is_ident_start(c) => self.lex_ident_or_keyword(),
                _ => {
                    // 其它字符：Step1 子集不支持，直接报错
                    self.bump_char();
                    return Err(self.err_at("UnexpectedChar", start_pos));
                }
            };

            // token 结束位置：注意 `mark()` 取的是“当前扫描指针”，所以 end 是开区间
            let end_pos = self.mark();
            tokens.push(Token {
                kind,
                span: Span::new_with_line_col(
                    start_pos.offset,
                    end_pos.offset,
                    start_pos.line,
                    start_pos.col,
                    end_pos.line,
                    end_pos.col,
                ),
            });
        }

        Ok(tokens)
    }

    /// 跳过空白与注释。
    ///
    /// - 空白：` ` `\t` `\r` `\n`
    /// - 单行注释：`// ... \n`
    /// - 块注释：`/* ... */`（这里额外支持，便于写测试/样例；不影响 Step1 目标）
    fn skip_ws_and_comments(&mut self) -> Result<(), Error> {
        loop {
            let mut progressed = false;
            while let Some(ch) = self.peek_char() {
                if ch == ' ' || ch == '\t' || ch == '\r' || ch == '\n' {
                    self.bump_char();
                    progressed = true;
                } else {
                    break;
                }
            }

            if self.peek_is("//") {
                self.bump_str("//");
                while let Some(ch) = self.peek_char() {
                    if ch == '\n' {
                        break;
                    }
                    self.bump_char();
                }
                continue;
            }

            if self.peek_is("/*") {
                let start = self.mark();
                self.bump_str("/*");
                while !self.is_eof() && !self.peek_is("*/") {
                    self.bump_char();
                }
                if self.peek_is("*/") {
                    self.bump_str("*/");
                } else {
                    return Err(self.err_at("UnterminatedBlockComment", start));
                }
                continue;
            }

            if !progressed {
                break;
            }
        }

        Ok(())
    }

    /// 读取连续数字，解析为 i32。
    fn lex_number(&mut self) -> Result<TokenKind, Error> {
        let start = self.mark();
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if ch.is_ascii_digit() {
                s.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }
        match s.parse::<i32>() {
            Ok(n) => Ok(TokenKind::Number(n)),
            Err(_) => Err(self.err_at("InvalidNumber", start)),
        }
    }

    /// 读取双引号字符串：`"..."`。
    ///
    /// 支持少量转义：`\"`, `\\`, `\n`, `\t`, `\r`。
    /// 如果遇到换行或 EOF 还没闭合，则报 `UnterminatedString`。
    fn lex_string(&mut self) -> Result<TokenKind, Error> {
        let start = self.mark();
        // 消费开头的 `"`
        self.bump_char();

        let mut out = String::new();
        while let Some(ch) = self.peek_char() {
            match ch {
                '"' => {
                    // 消费结尾的 `"`
                    self.bump_char();
                    return Ok(TokenKind::String(out));
                }
                '\n' => {
                    return Err(self.err_at("UnterminatedString", start));
                }
                '\\' => {
                    // 处理转义序列：先吃掉 `\`，再读一个字符作为转义目标
                    self.bump_char();
                    let esc = self
                        .peek_char()
                        .ok_or_else(|| self.err_at("UnterminatedString", start))?;
                    match esc {
                        '"' => {
                            out.push('"');
                            self.bump_char();
                        }
                        '\\' => {
                            out.push('\\');
                            self.bump_char();
                        }
                        'n' => {
                            out.push('\n');
                            self.bump_char();
                        }
                        't' => {
                            out.push('\t');
                            self.bump_char();
                        }
                        'r' => {
                            out.push('\r');
                            self.bump_char();
                        }
                        _ => {
                            out.push(esc);
                            self.bump_char();
                        }
                    }
                }
                _ => {
                    out.push(ch);
                    self.bump_char();
                }
            }
        }

        Err(self.err_at("UnterminatedString", start))
    }

    /// 读取标识符，并在此处做“关键字识别”。
    fn lex_ident_or_keyword(&mut self) -> TokenKind {
        let mut s = String::new();
        while let Some(ch) = self.peek_char() {
            if is_ident_continue(ch) {
                s.push(ch);
                self.bump_char();
            } else {
                break;
            }
        }

        match s.as_str() {
            "let" => TokenKind::KwLet,
            "const" => TokenKind::KwConst,
            "function" => TokenKind::KwFunction,
            "if" => TokenKind::KwIf,
            "else" => TokenKind::KwElse,
            "while" => TokenKind::KwWhile,
            "return" => TokenKind::KwReturn,
            "true" => TokenKind::KwTrue,
            "false" => TokenKind::KwFalse,
            _ => TokenKind::Ident(s),
        }
    }

    /// 是否到达输入末尾。
    fn is_eof(&self) -> bool {
        self.byte_pos >= self.src.len()
    }

    /// 查看当前字符（不消费）。
    fn peek_char(&self) -> Option<char> {
        self.src[self.byte_pos..].chars().next()
    }

    /// 消费一个字符，并同步更新 byte offset 与 line/col。
    fn bump_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        let len = ch.len_utf8();
        self.byte_pos += len;
        if ch == '\n' {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        Some(ch)
    }

    /// 如果下一个字符等于 expected，就消费它并返回 true；否则不动并返回 false。
    fn try_bump(&mut self, expected: char) -> bool {
        match self.peek_char() {
            Some(ch) if ch == expected => {
                self.bump_char();
                true
            }
            _ => false,
        }
    }

    /// 判断当前位置是否以某个字符串开头（用于注释 `//`、`/*`、`*/` 等）。
    fn peek_is(&self, s: &str) -> bool {
        self.src[self.byte_pos..].starts_with(s)
    }

    /// 消费一个短字符串（例如 `//`、`/*`、`*/`）。
    fn bump_str(&mut self, s: &str) {
        for _ in 0..s.chars().count() {
            self.bump_char();
        }
    }

    /// 在某个位置构造一个错误（Span 的起止点都指向该位置）。
    fn err_at(&self, code: &'static str, pos: Mark) -> Error {
        let span = Span::new_with_line_col(
            pos.offset,
            pos.offset,
            pos.line,
            pos.col,
            pos.line,
            pos.col,
        );
        Error::new(code, span)
    }

    /// 记录当前扫描指针的位置（byte offset + line/col）。
    fn mark(&self) -> Mark {
        Mark {
            offset: self.byte_pos,
            line: self.line,
            col: self.col,
        }
    }
}

/// 记录 Lexer 扫描指针的位置（内部使用）。
#[derive(Clone, Copy)]
struct Mark {
    offset: usize,
    line: usize,
    col: usize,
}

/// 标识符首字符规则：字母或 `_`。
fn is_ident_start(ch: char) -> bool {
    ch == '_' || ch.is_ascii_alphabetic()
}

/// 标识符后续字符规则：字母/数字/`_`。
fn is_ident_continue(ch: char) -> bool {
    is_ident_start(ch) || ch.is_ascii_digit()
}
