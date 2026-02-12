use crate::ast::{
    AssignStmt, BinaryExpr, BinaryOp, Callee, CallExpr, Expr, Literal, Program, Stmt, UnaryExpr,
    UnaryOp, VarDecl,
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
        Stmt::Assign(a) => gen_assign(a),
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
