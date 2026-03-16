use std::env;
use std::io::Write;
use std::process::{Command, Stdio};

pub mod ast;
pub mod codegen;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod span;

/// crate 的模块导出。
///
/// 小白视角可以这样理解：
/// - `src/lib.rs`：库（library）的“门面”，把内部模块组织好并对外暴露 API。
/// - `src/main.rs`：可执行程序（binary）的入口，通常只负责 I/O（读文件、写文件、打印错误）。
///
/// 这样拆分的好处：
/// - 测试更方便：tests/ 更像“外部用户”，只调用 lib 暴露的函数。
/// - 复用更容易：未来其它 Rust 项目也能直接依赖这个库。
pub use ast::{
    Callee, CallExpr, Expr, FuncDecl, Literal, Param, Program, Stmt, TypeAnn, VarDecl,
};
pub use error::Error;
pub use lexer::{lex, Token, TokenKind};
pub use parser::parse as parse_tokens;
pub use span::Span;

/// 辅助函数：直接从源代码解析出 Program AST。
///
/// 这对测试 Parser 很方便：不需要手动先调用 lex()。
pub fn parse_program(src: &str) -> Result<Program, Error> {
    let tokens = lex(src)?;
    parse_tokens(&tokens)
}

/// 编译入口：把 ArkTS 子集源码编译成 Rust 源码字符串。
///
/// 目前 Step3 的流水线是：
/// 1. Lexer：`src` -> `Vec<Token>`
/// 2. Parser：`Vec<Token>` -> `Program` AST
/// 3. CodeGen：`Program` -> Rust 源码字符串
///
/// 注意：这一步的“compile”只生成 Rust 源码，不会自动调用 rustc 去编译。
pub fn compile(src: &str) -> Result<String, Error> {
    let tokens = lex(src)?;
    let program = parse_tokens(&tokens)?;
    codegen::generate(&program)
}

pub fn compile_with_ai(src: &str) -> Result<String, Error> {
    match compile(src) {
        Ok(rust) => Ok(rust),
        Err(e) => match ai_fallback(src, &e) {
            Ok(rust) => Ok(rust),
            Err(_) => Err(e),
        },
    }
}

fn ai_fallback(src: &str, err: &Error) -> Result<String, Error> {
    eprintln!("[Compiler] Rule-based compilation failed: {}. Falling back to AI...", err);
    // 默认使用内置的桥接脚本路径，用户仍可以通过环境变量覆盖
    let cmd = match env::var("ARKTS2RUST_AI_CMD") {
        Ok(v) if !v.trim().is_empty() => v,
        _ => "python3 tools/ai_bridge.py".to_string(),
    };
    let prompt = build_ai_prompt(src, err);
    match run_ai_command(&cmd, &prompt) {
        Ok(out) if !out.trim().is_empty() => Ok(out),
        _ => Err(Error::new("AIFallbackFailed", Span::default())),
    }
}

fn build_ai_prompt(src: &str, err: &Error) -> String {
    let mut out = String::new();
    out.push_str("你是 ArkTS 到 Rust 的转换器。\n");
    out.push_str("编译器在规则转换阶段失败，请直接输出 Rust 源码。\n");
    out.push_str("只输出 Rust 代码，不要解释。\n\n");
    out.push_str("【ArkTS 源码】\n");
    out.push_str(src);
    out.push_str("\n\n【错误信息】\n");
    out.push_str(&format!("{}", err));
    out
}

fn run_ai_command(cmd: &str, input: &str) -> Result<String, Error> {
    let mut parts = cmd.split_whitespace();
    let program = match parts.next() {
        Some(p) => p,
        None => return Err(Error::new("AIFallbackInvalidCmd", Span::default())),
    };
    let args: Vec<&str> = parts.collect();
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(|_| Error::new("AIFallbackSpawnFailed", Span::default()))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input.as_bytes())
            .map_err(|_| Error::new("AIFallbackWriteFailed", Span::default()))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|_| Error::new("AIFallbackExecFailed", Span::default()))?;
    if !output.status.success() {
        return Err(Error::new("AIFallbackNonZeroExit", Span::default()));
    }
    String::from_utf8(output.stdout)
        .map_err(|_| Error::new("AIFallbackInvalidUtf8", Span::default()))
}
