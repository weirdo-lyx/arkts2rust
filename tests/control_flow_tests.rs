use std::fs;
use std::path::PathBuf;
use std::process::Command;

use arkts2rust::{compile, parse_program};

fn assert_codegen(src: &str, expected: &str) {
    let got = compile(src).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn if_else_basic_assign() {
    assert_codegen(
        "if (true) x=1; else x=2;",
        "fn main() {\n    if true {\n        x = 1i32;\n    } else {\n        x = 2i32;\n    }\n}\n",
    );
}

#[test]
fn if_else_with_block_branches() {
    assert_codegen(
        "if (false) { x=1; y=2; } else { x=3; }",
        "fn main() {\n    if false {\n        x = 1i32;\n        y = 2i32;\n    } else {\n        x = 3i32;\n    }\n}\n",
    );
}

#[test]
fn if_else_condition_precedence() {
    assert_codegen(
        "if (1<2==true) return; else return;",
        "fn main() {\n    if 1i32 < 2i32 == true {\n        return;\n    } else {\n        return;\n    }\n}\n",
    );
}

#[test]
fn if_else_nested_if_in_else() {
    assert_codegen(
        "if (false) return; else if (true) return; else return;",
        "fn main() {\n    if false {\n        return;\n    } else {\n        if true {\n            return;\n        } else {\n            return;\n        }\n    }\n}\n",
    );
}

#[test]
fn if_else_then_single_stmt_is_wrapped_in_rust_block() {
    assert_codegen(
        "if (true) console.log(1); else console.log(2);",
        "fn main() {\n    if true {\n        println!(\"{:?}\", 1i32);\n    } else {\n        println!(\"{:?}\", 2i32);\n    }\n}\n",
    );
}

#[test]
fn if_else_return_with_value_is_early_exit() {
    assert_codegen(
        "if (true) return 1; else return 2;",
        "fn main() {\n    if true {\n        let _ = 1i32;\n        return;\n    } else {\n        let _ = 2i32;\n        return;\n    }\n}\n",
    );
}

#[test]
fn while_false_empty_block() {
    assert_codegen(
        "while (false) { }",
        "fn main() {\n    while false {\n    }\n}\n",
    );
}

#[test]
fn while_comparison_updates_var() {
    assert_codegen(
        "let x=0; while (x<3) { x=x+1; }",
        "fn main() {\n    let mut x = 0i32;\n    while x < 3i32 {\n        x = x + 1i32;\n    }\n}\n",
    );
}

#[test]
fn while_body_can_be_single_stmt() {
    assert_codegen(
        "let x=0; while (x<1) x=x+1;",
        "fn main() {\n    let mut x = 0i32;\n    while x < 1i32 {\n        x = x + 1i32;\n    }\n}\n",
    );
}

#[test]
fn while_nested_in_block() {
    assert_codegen(
        "{ while (true) { return; } }",
        "fn main() {\n    {\n        while true {\n            return;\n        }\n    }\n}\n",
    );
}

#[test]
fn error_if_missing_else() {
    let err = parse_program("if (true) x=1;").expect_err("missing else should error");
    assert_eq!(err.code, "MissingElse");
}

#[test]
fn error_condition_must_be_bool() {
    let err = parse_program("if (1) x=1; else x=2;").expect_err("truthy is not allowed");
    assert_eq!(err.code, "ConditionMustBeBool");
}

#[test]
fn error_while_condition_must_be_bool() {
    let err = parse_program("while (1+2) { }").expect_err("arith expression is not bool");
    assert_eq!(err.code, "ConditionMustBeBool");
}

#[test]
fn generated_rust_can_compile_with_control_flow() {
    let src = r#"
let x=0;
while (x<3) { x=x+1; }
if (x==3) { console.log("ok"); } else { console.log("bad"); }
return;
"#;

    let rust = compile(src).unwrap();

    if Command::new("rustc").arg("--version").output().is_err() {
        return;
    }

    let mut path: PathBuf = std::env::temp_dir();
    let file_name = format!(
        "arkts2rust_step5_{}_{}.rs",
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
