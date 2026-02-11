use arkts2rust::{compile, Span};

#[test]
fn step0_compile_smoke() {
    let err = compile("let x = 1;").expect_err("Step0 should be a placeholder implementation");
    assert_eq!(err.code, "NotImplemented");
    assert_eq!(err.span, Span::default());
}
