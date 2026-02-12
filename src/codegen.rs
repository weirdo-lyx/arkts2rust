use crate::ast::{Callee, CallExpr, Expr, Literal, Program, Stmt, VarDecl};
use crate::error::Error;
use crate::span::Span;

/// CodeGen 的对外入口：把 AST（Program）生成 Rust 源码字符串。
///
/// 设计说明（Step3 子集）：
/// - 只处理 Step2 的最小 AST：变量声明 + `console.log(literal)`。
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
    out.push_str("fn main() {\n");
    for stmt in &program.stmts {
        out.push_str("    ");
        out.push_str(&gen_stmt(stmt)?);
        out.push('\n');
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
    match stmt {
        Stmt::VarDecl(v) => gen_var_decl(v),
        Stmt::ExprStmt(e) => {
            let expr = gen_expr(e)?;
            Ok(format!("{expr};"))
        }
    }
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
    let init = gen_literal_expr(&v.init);
    Ok(format!("{keyword} {} = {init};", v.name))
}

/// 生成表达式。
///
/// Step3 只支持：
/// - 字面量（Literal）
/// - console.log 调用（Call）
pub fn gen_expr(expr: &Expr) -> Result<String, Error> {
    match expr {
        Expr::Literal(lit) => Ok(gen_literal_expr(lit)),
        Expr::Call(call) => gen_call(call),
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
