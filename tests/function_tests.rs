use std::fs;
use std::path::PathBuf;
use std::process::Command;

use arkts2rust::ast::{
    BlockStmt, Expr, FuncDecl, Param, Program, Stmt, TypeAnn,
};
use arkts2rust::{compile, parse_program};

fn program(funcs: Vec<FuncDecl>, stmts: Vec<Stmt>) -> Program {
    Program { funcs, stmts }
}

fn block(stmts: Vec<Stmt>) -> BlockStmt {
    BlockStmt { stmts }
}

fn ident(s: &str) -> Expr {
    Expr::Ident(s.to_string())
}

#[test]
fn parse_function_with_types() {
    let p = parse_program("function add(a: number, b: number): number { return a+b; }").unwrap();
    assert_eq!(
        p,
        program(
            vec![FuncDecl {
                name: "add".into(),
                params: vec![
                    Param {
                        name: "a".into(),
                        ty: Some(TypeAnn::Number),
                    },
                    Param {
                        name: "b".into(),
                        ty: Some(TypeAnn::Number),
                    },
                ],
                ret_type: Some(TypeAnn::Number),
                body: block(vec![Stmt::Return(arkts2rust::ast::ReturnStmt {
                    value: Some(Expr::Binary(arkts2rust::ast::BinaryExpr {
                        op: arkts2rust::ast::BinaryOp::Add,
                        left: Box::new(ident("a")),
                        right: Box::new(ident("b")),
                    })),
                })]),
            }],
            vec![]
        )
    );
}

#[test]
fn parse_function_without_types() {
    let p = parse_program("function f(a, b) { return a+b; }").unwrap();
    assert_eq!(p.funcs.len(), 1);
    let f = &p.funcs[0];
    assert_eq!(f.name, "f");
    assert_eq!(f.params.len(), 2);
    assert_eq!(f.params[0].ty, None);
    assert_eq!(f.ret_type, None);
}

#[test]
fn codegen_outputs_functions_before_main() {
    let rust = compile("function id(a: number): number { return a; } id(1);").unwrap();
    assert!(
        rust.starts_with("fn id(a: i32) -> i32"),
        "expected function first, got:\n{rust}"
    );
    assert!(rust.contains("\nfn main() {\n"));
}

#[test]
fn codegen_default_param_type_is_i32() {
    let rust = compile("function id(a): number { return a; } id(1);").unwrap();
    assert!(rust.starts_with("fn id(a: i32) -> i32"));
}

#[test]
fn codegen_void_return_emits_return() {
    let rust = compile("function f(): void { return; } f();").unwrap();
    assert!(rust.contains("fn f() {\n    return;\n}\n"));
}

#[test]
fn codegen_void_return_value_is_dropped() {
    let rust = compile("function f(): void { return 1; } f();").unwrap();
    assert!(rust.contains("let _ = 1i32;"));
    assert!(rust.contains("return;"));
}

#[test]
fn codegen_requires_return_value_for_non_void() {
    let err = compile("function f(): number { return; }").expect_err("non-void return needs value");
    assert_eq!(err.code, "ReturnValueRequired");
}

#[test]
fn error_unknown_type() {
    let err = parse_program("function f(a: xyz): number { return 1; }")
        .expect_err("unknown type should error");
    assert_eq!(err.code, "UnknownType");
    assert_eq!(err.span.start_line, 1);
}

#[test]
fn error_expected_block_after_signature() {
    let err = parse_program("function f(): number return 1;").expect_err("missing block should error");
    assert_eq!(err.code, "ExpectedBlock");
}

#[test]
fn function_call_inside_return() {
    let rust = compile(
        "function add(a:number,b:number): number { return a+b; } \
         function main2(): number { return add(1,2); } \
         main2();",
    )
    .unwrap();
    assert!(rust.contains("fn main2() -> i32"));
    assert!(rust.contains("return add(1i32, 2i32);"));
}

#[test]
fn generated_rust_with_functions_can_compile() {
    let src = r#"
function add(a:number, b:number): number { return a+b; }
function always(): boolean { return true; }
add(1,2);
if (always()) { return; } else { return; }
"#;

    let rust = compile(src).unwrap();

    if Command::new("rustc").arg("--version").output().is_err() {
        return;
    }

    let mut path: PathBuf = std::env::temp_dir();
    let file_name = format!(
        "arkts2rust_step6_{}_{}.rs",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    );
    path.push(file_name);

    let mut exe_path = path.clone();
    exe_path.set_extension("");

    fs::write(&path, rust).unwrap();
    let out = Command::new("rustc")
        .arg(&path)
        .arg("-o")
        .arg(&exe_path)
        .output()
        .unwrap();
    let _ = fs::remove_file(&path);
    let _ = fs::remove_file(&exe_path);

    assert!(
        out.status.success(),
        "rustc failed: {}\n{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn codegen_multiple_functions_and_top_level_stmts_go_to_main() {
    let rust = compile(
        "function a(): void { return; } \
         function b(x: number): number { return x; } \
         a(); b(1);",
    )
    .unwrap();

    let a_pos = rust.find("fn a()").unwrap();
    let b_pos = rust.find("fn b(").unwrap();
    let main_pos = rust.find("fn main()").unwrap();
    assert!(a_pos < b_pos && b_pos < main_pos);
    assert!(rust.contains("a();"));
    assert!(rust.contains("b(1i32);"));
}
