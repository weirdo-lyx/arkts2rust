use arkts2rust::{
    parse_program, parse_tokens, Callee, CallExpr, Expr, Literal, Program, Stmt, TokenKind,
    VarDecl,
};

fn program(stmts: Vec<Stmt>) -> Program {
    Program { stmts }
}

#[test]
fn parse_let_number() {
    let p = parse_program("let x = 1;").unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::VarDecl(VarDecl {
            is_const: false,
            name: "x".into(),
            init: Literal::Number(1),
        })])
    );
}

#[test]
fn parse_const_string() {
    let p = parse_program(r#"const s = "hi";"#).unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::VarDecl(VarDecl {
            is_const: true,
            name: "s".into(),
            init: Literal::String("hi".into()),
        })])
    );
}

#[test]
fn parse_let_bool_true() {
    let p = parse_program("let ok = true;").unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::VarDecl(VarDecl {
            is_const: false,
            name: "ok".into(),
            init: Literal::Bool(true),
        })])
    );
}

#[test]
fn parse_console_log_number() {
    let p = parse_program("console.log(1);").unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::ExprStmt(Expr::Call(CallExpr {
            callee: Callee::ConsoleLog,
            args: vec![Expr::Literal(Literal::Number(1))],
        }))])
    );
}

#[test]
fn parse_console_log_string() {
    let p = parse_program(r#"console.log("a");"#).unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::ExprStmt(Expr::Call(CallExpr {
            callee: Callee::ConsoleLog,
            args: vec![Expr::Literal(Literal::String("a".into()))],
        }))])
    );
}

#[test]
fn parse_multiple_stmts() {
    let p = parse_program("let x = 1; console.log(x);").unwrap_err();
    assert_eq!(p.code, "ExpectedLiteral");
}

#[test]
fn parse_program_with_comments_and_newlines() {
    let src = r#"
// c1
let x = 1;
/* c2 */ console.log(true);
"#;
    let p = parse_program(src).unwrap();
    assert_eq!(
        p,
        program(vec![
            Stmt::VarDecl(VarDecl {
                is_const: false,
                name: "x".into(),
                init: Literal::Number(1),
            }),
            Stmt::ExprStmt(Expr::Call(CallExpr {
                callee: Callee::ConsoleLog,
                args: vec![Expr::Literal(Literal::Bool(true))],
            })),
        ])
    );
}

#[test]
fn error_missing_semicolon() {
    let err = parse_program("let x = 1").expect_err("missing semicolon should error");
    assert_eq!(err.code, "MissingSemicolon");
    assert_eq!(err.span.start_line, 1);
}

#[test]
fn error_missing_rparen() {
    let err = parse_program("console.log(1;").expect_err("missing rparen should error");
    assert_eq!(err.code, "MissingRParen");
    assert_eq!(err.span.start_line, 1);
}

#[test]
fn error_unknown_structure() {
    let err = parse_program("foo(1);").expect_err("unknown call should error");
    assert_eq!(err.code, "UnknownStructure");
    assert_eq!(err.span.start_line, 1);
    assert_eq!(err.span.start_col, 1);
}

#[test]
fn parse_from_tokens_directly() {
    let tokens = arkts2rust::lex("let x = 1;").unwrap();
    let p = parse_tokens(&tokens).unwrap();
    assert_eq!(
        p,
        program(vec![Stmt::VarDecl(VarDecl {
            is_const: false,
            name: "x".into(),
            init: Literal::Number(1),
        })])
    );
}

#[test]
fn lexer_produces_dot_token_for_console_log() {
    let kinds: Vec<TokenKind> = arkts2rust::lex("console.log(1);")
        .unwrap()
        .into_iter()
        .map(|t| t.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::Ident("console".into()),
            TokenKind::Dot,
            TokenKind::Ident("log".into()),
            TokenKind::LParen,
            TokenKind::Number(1),
            TokenKind::RParen,
            TokenKind::Semicolon,
        ]
    );
}

