use arkts2rust::{lex, Error, TokenKind};

fn kinds(src: &str) -> Result<Vec<TokenKind>, Error> {
    Ok(lex(src)?.into_iter().map(|t| t.kind).collect())
}

#[test]
fn lex_keywords_vs_ident() {
    let ks = kinds("let lettuce const constant if iff true truth false falsy").unwrap();
    assert_eq!(
        ks,
        vec![
            TokenKind::KwLet,
            TokenKind::Ident("lettuce".into()),
            TokenKind::KwConst,
            TokenKind::Ident("constant".into()),
            TokenKind::KwIf,
            TokenKind::Ident("iff".into()),
            TokenKind::KwTrue,
            TokenKind::Ident("truth".into()),
            TokenKind::KwFalse,
            TokenKind::Ident("falsy".into()),
        ]
    );
}

#[test]
fn lex_number() {
    let ks = kinds("0 12 340").unwrap();
    assert_eq!(
        ks,
        vec![TokenKind::Number(0), TokenKind::Number(12), TokenKind::Number(340)]
    );
}

#[test]
fn lex_string_basic() {
    let ks = kinds(r#""hello""#).unwrap();
    assert_eq!(ks, vec![TokenKind::String("hello".into())]);
}

#[test]
fn lex_string_escape() {
    let ks = kinds(r#""a\"b\\c\n""#).unwrap();
    assert_eq!(ks, vec![TokenKind::String("a\"b\\c\n".into())]);
}

#[test]
fn lex_punctuations() {
    let ks = kinds("( ) { } , ;").unwrap();
    assert_eq!(
        ks,
        vec![
            TokenKind::LParen,
            TokenKind::RParen,
            TokenKind::LBrace,
            TokenKind::RBrace,
            TokenKind::Comma,
            TokenKind::Semicolon,
        ]
    );
}

#[test]
fn lex_operators_single() {
    let ks = kinds("+ - * / % < > ! =").unwrap();
    assert_eq!(
        ks,
        vec![
            TokenKind::Plus,
            TokenKind::Minus,
            TokenKind::Star,
            TokenKind::Slash,
            TokenKind::Percent,
            TokenKind::Lt,
            TokenKind::Gt,
            TokenKind::Not,
            TokenKind::Eq,
        ]
    );
}

#[test]
fn lex_operators_multi() {
    let ks = kinds("== != <= >= && ||").unwrap();
    assert_eq!(
        ks,
        vec![
            TokenKind::EqEq,
            TokenKind::NotEq,
            TokenKind::LtEq,
            TokenKind::GtEq,
            TokenKind::AndAnd,
            TokenKind::OrOr,
        ]
    );
}

#[test]
fn skip_whitespace_and_line_comment() {
    let ks = kinds(
        r#"
let x // comment
const y
"#,
    )
    .unwrap();
    assert_eq!(
        ks,
        vec![TokenKind::KwLet, TokenKind::Ident("x".into()), TokenKind::KwConst, TokenKind::Ident("y".into())]
    );
}

#[test]
fn skip_block_comment() {
    let ks = kinds("let/* hi */x").unwrap();
    assert_eq!(ks, vec![TokenKind::KwLet, TokenKind::Ident("x".into())]);
}

#[test]
fn error_unexpected_char() {
    let err = lex("@").expect_err("should fail on illegal character");
    assert_eq!(err.code, "UnexpectedChar");
    assert_eq!(err.span.start_line, 1);
    assert_eq!(err.span.start_col, 1);
}

#[test]
fn error_unterminated_string() {
    let err = lex("\"abc").expect_err("should fail on unterminated string");
    assert_eq!(err.code, "UnterminatedString");
    assert_eq!(err.span.start_line, 1);
    assert_eq!(err.span.start_col, 1);
}

#[test]
fn span_line_col_across_newline() {
    let tokens = lex("let\nx").unwrap();
    assert_eq!(tokens[0].span.start_line, 1);
    assert_eq!(tokens[0].span.start_col, 1);
    assert_eq!(tokens[1].span.start_line, 2);
    assert_eq!(tokens[1].span.start_col, 1);
}
