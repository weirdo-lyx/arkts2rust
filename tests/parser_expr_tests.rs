use arkts2rust::ast::{
    AssignStmt, BinaryExpr, BinaryOp, Callee, CallExpr, Expr, Literal, Stmt, UnaryExpr, UnaryOp,
};
use arkts2rust::parse_program;

fn stmt(src: &str) -> Stmt {
    let p = parse_program(src).unwrap();
    assert_eq!(p.stmts.len(), 1);
    p.stmts.into_iter().next().unwrap()
}

fn lit_i(n: i32) -> Expr {
    Expr::Literal(Literal::Number(n))
}

fn lit_b(b: bool) -> Expr {
    Expr::Literal(Literal::Bool(b))
}

fn ident(name: &str) -> Expr {
    Expr::Ident(name.to_string())
}

fn unary(op: UnaryOp, expr: Expr) -> Expr {
    Expr::Unary(UnaryExpr {
        op,
        expr: Box::new(expr),
    })
}

fn binary(op: BinaryOp, left: Expr, right: Expr) -> Expr {
    Expr::Binary(BinaryExpr {
        op,
        left: Box::new(left),
        right: Box::new(right),
    })
}

fn group(expr: Expr) -> Expr {
    Expr::Group(Box::new(expr))
}

fn call(name: &str, args: Vec<Expr>) -> Expr {
    Expr::Call(CallExpr {
        callee: Callee::Ident(name.to_string()),
        args,
    })
}

#[test]
fn precedence_mul_over_add() {
    let s = stmt("1+2*3;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::Add,
            lit_i(1),
            binary(BinaryOp::Mul, lit_i(2), lit_i(3))
        ))
    );
}

#[test]
fn precedence_parens_override() {
    let s = stmt("(1+2)*3;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::Mul,
            group(binary(BinaryOp::Add, lit_i(1), lit_i(2))),
            lit_i(3)
        ))
    );
}

#[test]
fn left_associative_additive() {
    let s = stmt("1-2-3;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::Sub,
            binary(BinaryOp::Sub, lit_i(1), lit_i(2)),
            lit_i(3)
        ))
    );
}

#[test]
fn unary_neg_binds_tight() {
    let s = stmt("-1*2;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::Mul,
            unary(UnaryOp::Neg, lit_i(1)),
            lit_i(2)
        ))
    );
}

#[test]
fn unary_not_binds_tight() {
    let s = stmt("!true==false;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::EqEq,
            unary(UnaryOp::Not, lit_b(true)),
            lit_b(false)
        ))
    );
}

#[test]
fn comparison_binds_tighter_than_equality() {
    let s = stmt("1<2==true;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::EqEq,
            binary(BinaryOp::Lt, lit_i(1), lit_i(2)),
            lit_b(true)
        ))
    );
}

#[test]
fn and_binds_tighter_than_or() {
    let s = stmt("a&&b||c;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::OrOr,
            binary(BinaryOp::AndAnd, ident("a"), ident("b")),
            ident("c")
        ))
    );
}

#[test]
fn parens_in_boolean_expr() {
    let s = stmt("a&&(b||c);");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(
            BinaryOp::AndAnd,
            ident("a"),
            group(binary(BinaryOp::OrOr, ident("b"), ident("c")))
        ))
    );
}

#[test]
fn call_simple() {
    let s = stmt("f(1,2);");
    assert_eq!(s, Stmt::ExprStmt(call("f", vec![lit_i(1), lit_i(2)])));
}

#[test]
fn call_has_higher_precedence_than_add() {
    let s = stmt("f(1)+2;");
    assert_eq!(
        s,
        Stmt::ExprStmt(binary(BinaryOp::Add, call("f", vec![lit_i(1)]), lit_i(2)))
    );
}

#[test]
fn call_args_can_be_expressions() {
    let s = stmt("f(1+2*3,-4);");
    assert_eq!(
        s,
        Stmt::ExprStmt(call(
            "f",
            vec![
                binary(
                    BinaryOp::Add,
                    lit_i(1),
                    binary(BinaryOp::Mul, lit_i(2), lit_i(3))
                ),
                unary(UnaryOp::Neg, lit_i(4))
            ]
        ))
    );
}

#[test]
fn ident_reference_stmt() {
    let s = stmt("x;");
    assert_eq!(s, Stmt::ExprStmt(ident("x")));
}

#[test]
fn assign_stmt_basic() {
    let s = stmt("x=1+2*3;");
    assert_eq!(
        s,
        Stmt::Assign(AssignStmt {
            name: "x".into(),
            value: binary(
                BinaryOp::Add,
                lit_i(1),
                binary(BinaryOp::Mul, lit_i(2), lit_i(3))
            ),
        })
    );
}

#[test]
fn assign_stmt_with_call() {
    let s = stmt("x=f(1,2);");
    assert_eq!(
        s,
        Stmt::Assign(AssignStmt {
            name: "x".into(),
            value: call("f", vec![lit_i(1), lit_i(2)]),
        })
    );
}

#[test]
fn codegen_add_mul_no_extra_parens() {
    let rust = arkts2rust::compile("1+2*3;").unwrap();
    assert_eq!(rust, "fn main() {\n    1i32 + 2i32 * 3i32;\n}\n");
}

#[test]
fn codegen_parens_preserved() {
    let rust = arkts2rust::compile("(1+2)*3;").unwrap();
    assert_eq!(rust, "fn main() {\n    (1i32 + 2i32) * 3i32;\n}\n");
}

#[test]
fn error_missing_rparen_in_group() {
    let err = parse_program("(1+2;").expect_err("missing ')' should error");
    assert_eq!(err.code, "MissingRParen");
    assert_eq!(err.span.start_line, 1);
    assert_eq!(err.span.start_col, 5);
}

#[test]
fn error_expected_expr_after_operator() {
    let err = parse_program("1+;").expect_err("missing rhs should error");
    assert_eq!(err.code, "ExpectedExpr");
    assert_eq!(err.span.start_line, 1);
    assert_eq!(err.span.start_col, 3);
}

