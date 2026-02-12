use arkts2rust::compile;

#[test]
fn step0_compile_smoke() {
    let rust = compile("let x = 1;").unwrap();
    assert_eq!(rust, "fn main() {\n    let mut x = 1i32;\n}\n");
}
