use crate::ast::{
    AssignStmt, BinaryExpr, BinaryOp, BlockStmt, Callee, CallExpr, Expr, FuncDecl, IfStmt, Literal,
    Param, Program, ReturnStmt, Stmt, TypeAnn, UnaryExpr, UnaryOp, VarDecl, WhileStmt,
};
use crate::error::Error;
use crate::span::Span;

/// CodeGen 的对外入口：把 AST（Program）生成 Rust 源码字符串。
///
/// 设计说明（Step4 子集）：
/// - 只处理 Step2/Step4 的 AST：变量声明、赋值、表达式（含优先级）、函数调用。
/// - 生成“完整 Rust 程序”，因此总是输出 `fn main(){ ... }` 结构。
/// - 这里的输出是字符串，是否写入文件由 CLI（main.rs）负责。
pub fn generate(program: &Program) -> Result<String, Error> {
    Ok(gen_program(program)?)
}

/// 生成完整 Rust 程序。
///
/// 输出格式（固定）：
/// ```text
/// fn main() {
///     <stmt1>
///     <stmt2>
/// }
/// ```
///
/// 这里采用非常简单的缩进策略：每条语句前面统一加 4 个空格。
pub fn gen_program(program: &Program) -> Result<String, Error> {
    let mut out = String::new();
    for f in &program.funcs {
        out.push_str(&gen_func_decl(f)?);
        out.push('\n');
    }
    out.push_str("fn main() {\n");
    for stmt in &program.stmts {
        gen_stmt_into(&mut out, 1, ReturnCtx::Main, stmt)?;
    }
    out.push_str("}\n");
    Ok(out)
}

/// 生成单条语句。
///
/// 注意：AST 层面不保存分号；分号属于语法细节。
/// - 变量声明语句：内部会补 `;`
/// - 表达式语句：这里统一在表达式后补 `;`
pub fn gen_stmt(stmt: &Stmt) -> Result<String, Error> {
    let mut out = String::new();
    gen_stmt_into(&mut out, 0, ReturnCtx::Main, stmt)?;
    Ok(out.trim_end_matches('\n').to_string())
}

/// 生成变量声明。
///
/// 映射规则：
/// - ArkTS `let` -> Rust `let mut`
/// - ArkTS `const` -> Rust `let`
///
/// 例：
/// - `let x = 1;` -> `let mut x = 1i32;`
/// - `const s = "hi";` -> `let s = String::from("hi");`
fn gen_var_decl(v: &VarDecl) -> Result<String, Error> {
    let keyword = if v.is_const { "let" } else { "let mut" };
    let init = gen_expr(&v.init)?;
    Ok(format!("{keyword} {} = {init};", v.name))
}

/// 生成表达式。
///
/// 生成表达式（Step4：含一元/二元/括号/调用/标识符）。
///
/// 核心要求：生成的 Rust 表达式必须与 AST 的求值顺序一致。
/// 因此在必要时需要补括号（例如 `(1+2)*3` 不能生成 `1+2*3`）。
pub fn gen_expr(expr: &Expr) -> Result<String, Error> {
    gen_expr_bp(expr, 0)
}

fn gen_assign(a: &AssignStmt) -> Result<String, Error> {
    let value = gen_expr(&a.value)?;
    Ok(format!("{} = {value};", a.name))
}

fn gen_return(r: &ReturnStmt) -> Result<Vec<String>, Error> {
    match &r.value {
        None => Ok(vec!["return;".to_string()]),
        Some(v) => {
            let value = gen_expr(v)?;
            Ok(vec![format!("let _ = {value};"), "return;".to_string()])
        }
    }
}

fn gen_block_body(out: &mut String, indent: usize, ctx: ReturnCtx, stmt: &Stmt) -> Result<(), Error> {
    match stmt {
        Stmt::Block(b) => {
            for s in &b.stmts {
                gen_stmt_into(out, indent, ctx, s)?;
            }
            Ok(())
        }
        _ => gen_stmt_into(out, indent, ctx, stmt),
    }
}

fn gen_stmt_into(out: &mut String, indent: usize, ctx: ReturnCtx, stmt: &Stmt) -> Result<(), Error> {
    match stmt {
        Stmt::VarDecl(v) => {
            push_indent(out, indent);
            out.push_str(&gen_var_decl(v)?);
            out.push('\n');
            Ok(())
        }
        Stmt::Assign(a) => {
            push_indent(out, indent);
            out.push_str(&gen_assign(a)?);
            out.push('\n');
            Ok(())
        }
        Stmt::ExprStmt(e) => {
            push_indent(out, indent);
            out.push_str(&format!("{};", gen_expr(e)?));
            out.push('\n');
            Ok(())
        }
        Stmt::Return(r) => {
            for line in gen_return_ctx(ctx, r)? {
                push_indent(out, indent);
                out.push_str(&line);
                out.push('\n');
            }
            Ok(())
        }
        Stmt::Block(b) => {
            out.push_str(&gen_block_ctx(ctx, b, indent)?);
            Ok(())
        }
        Stmt::If(i) => {
            out.push_str(&gen_if_ctx(ctx, i, indent)?);
            Ok(())
        }
        Stmt::While(w) => {
            out.push_str(&gen_while_ctx(ctx, w, indent)?);
            Ok(())
        }
    }
}

fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("    ");
    }
}

#[derive(Clone, Copy)]
enum ReturnCtx {
    Main,
    Function(TypeAnn),
}

fn gen_return_ctx(ctx: ReturnCtx, r: &ReturnStmt) -> Result<Vec<String>, Error> {
    match ctx {
        ReturnCtx::Main => gen_return(r),
        ReturnCtx::Function(ret) => match ret {
            TypeAnn::Void => match &r.value {
                None => Ok(vec!["return;".to_string()]),
                Some(v) => {
                    let value = gen_expr(v)?;
                    Ok(vec![format!("let _ = {value};"), "return;".to_string()])
                }
            },
            _ => match &r.value {
                Some(v) => Ok(vec![format!("return {};", gen_expr(v)?)]),
                None => Err(Error::new("ReturnValueRequired", Span::default())),
            },
        },
    }
}

fn gen_block_ctx(ctx: ReturnCtx, b: &BlockStmt, indent: usize) -> Result<String, Error> {
    let mut out = String::new();
    push_indent(&mut out, indent);
    out.push_str("{\n");
    for s in &b.stmts {
        gen_stmt_into(&mut out, indent + 1, ctx, s)?;
    }
    push_indent(&mut out, indent);
    out.push_str("}\n");
    Ok(out)
}

fn gen_if_ctx(ctx: ReturnCtx, stmt: &IfStmt, indent: usize) -> Result<String, Error> {
    let cond = gen_expr(&stmt.cond)?;

    let mut out = String::new();
    push_indent(&mut out, indent);
    out.push_str("if ");
    out.push_str(&cond);
    out.push_str(" {\n");
    gen_block_body(&mut out, indent + 1, ctx, &stmt.then_branch)?;
    push_indent(&mut out, indent);
    out.push('}');

    if let Some(else_branch) = &stmt.else_branch {
        out.push_str(" else {\n");
        gen_block_body(&mut out, indent + 1, ctx, else_branch)?;
        push_indent(&mut out, indent);
        out.push_str("}\n");
    } else {
        out.push('\n');
    }
    Ok(out)
}

fn gen_while_ctx(ctx: ReturnCtx, stmt: &WhileStmt, indent: usize) -> Result<String, Error> {
    let cond = gen_expr(&stmt.cond)?;

    let mut out = String::new();
    push_indent(&mut out, indent);
    out.push_str("while ");
    out.push_str(&cond);
    out.push_str(" {\n");
    gen_block_body(&mut out, indent + 1, ctx, &stmt.body)?;
    push_indent(&mut out, indent);
    out.push_str("}\n");
    Ok(out)
}

fn gen_func_decl(f: &FuncDecl) -> Result<String, Error> {
    let ret = effective_ret_type(f);
    let mut params = Vec::new();
    for p in &f.params {
        params.push(gen_param(p));
    }

    let mut out = String::new();
    out.push_str("fn ");
    out.push_str(&f.name);
    out.push('(');
    out.push_str(&params.join(", "));
    out.push(')');
    if ret != TypeAnn::Void {
        out.push_str(" -> ");
        out.push_str(&rust_type(ret));
    }
    out.push_str(" {\n");
    for s in &f.body.stmts {
        gen_stmt_into(&mut out, 1, ReturnCtx::Function(ret), s)?;
    }
    out.push_str("}\n");
    Ok(out)
}

fn gen_param(p: &Param) -> String {
    let ty = p.ty.unwrap_or(TypeAnn::Number);
    format!("{}: {}", p.name, rust_type(ty))
}

fn rust_type(t: TypeAnn) -> String {
    match t {
        TypeAnn::Number => "i32".to_string(),
        TypeAnn::String => "String".to_string(),
        TypeAnn::Boolean => "bool".to_string(),
        TypeAnn::Void => "()".to_string(),
    }
}

fn effective_ret_type(f: &FuncDecl) -> TypeAnn {
    match f.ret_type {
        Some(t) => t,
        None => {
            if func_body_has_return_value(&f.body) {
                TypeAnn::Number
            } else {
                TypeAnn::Void
            }
        }
    }
}

fn func_body_has_return_value(b: &BlockStmt) -> bool {
    b.stmts.iter().any(stmt_has_return_value)
}

fn stmt_has_return_value(s: &Stmt) -> bool {
    match s {
        Stmt::Return(r) => r.value.is_some(),
        Stmt::Block(b) => b.stmts.iter().any(stmt_has_return_value),
        Stmt::If(i) => {
            stmt_has_return_value(&i.then_branch)
                || i.else_branch
                    .as_ref()
                    .map(|b| stmt_has_return_value(b))
                    .unwrap_or(false)
        }
        Stmt::While(w) => stmt_has_return_value(&w.body),
        _ => false,
    }
}

/// 生成函数调用表达式。
///
/// 映射规则：
/// - `console.log(e)` -> `println!("{:?}", e)`
///
/// 目前约束：只允许 1 个参数。
fn gen_call(call: &CallExpr) -> Result<String, Error> {
    match call.callee {
        Callee::ConsoleLog => {
            if call.args.len() != 1 {
                // AST 理论上不会出现这个情况（Step2 parser 固定生成一个参数）。
                // 这里的分支属于“防御式编程”：即使未来 AST 扩展，错误也能被捕获。
                return Err(Error::new("UnsupportedAst", Span::default()));
            }
            let arg = gen_expr(&call.args[0])?;
            Ok(format!("println!(\"{{:?}}\", {arg})"))
        }
        Callee::Ident(ref name) => {
            let mut args = Vec::new();
            for a in &call.args {
                args.push(gen_expr(a)?);
            }
            Ok(format!("{name}({})", args.join(", ")))
        }
    }
}

fn gen_expr_bp(expr: &Expr, parent_bp: u8) -> Result<String, Error> {
    // 这里用“表达式绑定强度（bp）”来决定是否加括号：
    // - 子表达式 bp < 父表达式 bp 时，必须加括号，避免 Rust 按自己的优先级重排。
    // - bp 数值越大，优先级越高（绑定越紧）。
    let (s, bp) = match expr {
        Expr::Literal(lit) => (gen_literal_expr(lit), 100),
        Expr::Ident(name) => (name.clone(), 100),
        Expr::Group(inner) => (format!("({})", gen_expr_bp(inner, 0)?), 100),
        Expr::Call(call) => (gen_call(call)?, 90),
        Expr::Unary(u) => (gen_unary(u)?, 80),
        Expr::Binary(b) => (gen_binary(b)?, binary_bp(b.op)),
    };

    if bp < parent_bp {
        Ok(format!("({s})"))
    } else {
        Ok(s)
    }
}

fn gen_unary(u: &UnaryExpr) -> Result<String, Error> {
    let op = match u.op {
        UnaryOp::Not => "!",
        UnaryOp::Neg => "-",
    };
    let rhs = gen_expr_bp(&u.expr, 80)?;
    Ok(format!("{op}{rhs}"))
}

fn gen_binary(b: &BinaryExpr) -> Result<String, Error> {
    let op = match b.op {
        BinaryOp::Add => "+",
        BinaryOp::Sub => "-",
        BinaryOp::Mul => "*",
        BinaryOp::Div => "/",
        BinaryOp::Mod => "%",
        BinaryOp::EqEq => "==",
        BinaryOp::NotEq => "!=",
        BinaryOp::Lt => "<",
        BinaryOp::LtEq => "<=",
        BinaryOp::Gt => ">",
        BinaryOp::GtEq => ">=",
        BinaryOp::AndAnd => "&&",
        BinaryOp::OrOr => "||",
    };

    let bp = binary_bp(b.op);
    let left = gen_expr_bp(&b.left, bp)?;
    let right = gen_expr_bp(&b.right, bp + 1)?;
    Ok(format!("{left} {op} {right}"))
}

fn binary_bp(op: BinaryOp) -> u8 {
    match op {
        BinaryOp::OrOr => 20,
        BinaryOp::AndAnd => 30,
        BinaryOp::EqEq | BinaryOp::NotEq => 40,
        BinaryOp::Lt | BinaryOp::LtEq | BinaryOp::Gt | BinaryOp::GtEq => 50,
        BinaryOp::Add | BinaryOp::Sub => 60,
        BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => 70,
    }
}

/// 把字面量转换为 Rust 表达式字符串。
///
/// 映射规则：
/// - number -> i32（通过 `1i32` 这种后缀强制类型，避免类型推断差异）
/// - string -> String（统一用 `String::from("...")`）
/// - boolean -> bool
fn gen_literal_expr(lit: &Literal) -> String {
    match lit {
        Literal::Number(n) => format!("{n}i32"),
        Literal::Bool(b) => b.to_string(),
        Literal::String(s) => format!("String::from(\"{}\")", escape_rust_string(s)),
    }
}

/// 将字符串内容转义为可以放进 Rust 字符串字面量 `"..."` 的形式。
///
/// 例如：源码里包含 `"` 或 `\` 时，需要变为 `\"`、`\\`。
/// 否则生成的 Rust 代码将无法编译。
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
