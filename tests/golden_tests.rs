use arkts2rust::compile;

fn assert_golden(src: &str, expected: &str) {
    let got = compile(src).unwrap();
    assert_eq!(got, expected);
}

#[test]
fn golden_let_number() {
    assert_golden("let x = 1;", "fn main() {\n    let mut x = 1i32;\n}\n");
}

#[test]
fn golden_const_string() {
    assert_golden(
        "const s = \"hi\";",
        "fn main() {\n    let s = String::from(\"hi\");\n}\n",
    );
}

#[test]
fn golden_let_bool_false() {
    assert_golden(
        "let ok = false;",
        "fn main() {\n    let mut ok = false;\n}\n",
    );
}

#[test]
fn golden_console_log_number() {
    assert_golden(
        "console.log(1);",
        "fn main() {\n    println!(\"{:?}\", 1i32);\n}\n",
    );
}

#[test]
fn golden_multi_stmts() {
    assert_golden(
        "let x = 1; console.log(\"a\");",
        "fn main() {\n    let mut x = 1i32;\n    println!(\"{:?}\", String::from(\"a\"));\n}\n",
    );
}

#[test]
fn golden_string_escape_quote_and_backslash() {
    assert_golden(
        "console.log(\"a\\\"b\\\\c\");",
        "fn main() {\n    println!(\"{:?}\", String::from(\"a\\\"b\\\\c\"));\n}\n",
    );
}
