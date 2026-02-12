use crate::ast::{Callee, CallExpr, Expr, Literal, Program, Stmt, VarDecl};
use crate::error::Error;
use crate::span::Span;

pub fn generate(program: &Program) -> Result<String, Error> {
    Ok(gen_program(program)?)
}

pub fn gen_program(program: &Program) -> Result<String, Error> {
    let mut out = String::new();
    out.push_str("fn main() {\n");
    for stmt in &program.stmts {
        out.push_str("    ");
        out.push_str(&gen_stmt(stmt)?);
        out.push('\n');
    }
    out.push_str("}\n");
    Ok(out)
}

pub fn gen_stmt(stmt: &Stmt) -> Result<String, Error> {
    match stmt {
        Stmt::VarDecl(v) => gen_var_decl(v),
        Stmt::ExprStmt(e) => {
            let expr = gen_expr(e)?;
            Ok(format!("{expr};"))
        }
    }
}

fn gen_var_decl(v: &VarDecl) -> Result<String, Error> {
    let keyword = if v.is_const { "let" } else { "let mut" };
    let init = gen_literal_expr(&v.init);
    Ok(format!("{keyword} {} = {init};", v.name))
}

pub fn gen_expr(expr: &Expr) -> Result<String, Error> {
    match expr {
        Expr::Literal(lit) => Ok(gen_literal_expr(lit)),
        Expr::Call(call) => gen_call(call),
    }
}

fn gen_call(call: &CallExpr) -> Result<String, Error> {
    match call.callee {
        Callee::ConsoleLog => {
            if call.args.len() != 1 {
                return Err(Error::new("UnsupportedAst", Span::default()));
            }
            let arg = gen_expr(&call.args[0])?;
            Ok(format!("println!(\"{{:?}}\", {arg})"))
        }
    }
}

fn gen_literal_expr(lit: &Literal) -> String {
    match lit {
        Literal::Number(n) => format!("{n}i32"),
        Literal::Bool(b) => b.to_string(),
        Literal::String(s) => format!("String::from(\"{}\")", escape_rust_string(s)),
    }
}

fn escape_rust_string(s: &str) -> String {
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}
