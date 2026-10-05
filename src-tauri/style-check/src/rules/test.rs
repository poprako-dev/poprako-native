use std::error::Error;

use crate::rules::check;

#[test]
fn rejects_each_documented_syntax_violation() -> Result<(), Box<dyn Error>> {
    for (source, message) in [
        ("pub(crate) struct A;", "scoped visibility"),
        ("fn a() { if true {} else {} }", "instead of else"),
        ("fn a() { unsafe {} }", "unsafe is forbidden"),
        ("use a::{b::c,d};", "leaf names"),
        ("fn a() { let tx = 1; }", "send/recv"),
        ("struct A; fn b() {} impl A {}", "immediately follow"),
        ("fn a() { b(); } fn b() {}", "defined before"),
        ("fn a() { let a=1; let b=2; }", "blank line"),
    ] {
        let parsed = syn::parse_file(source)?;

        assert!(
            check(&parsed).iter().any(|error| error.contains(message)),
            "missing rule: {message}"
        );
    }

    Ok(())
}

#[test]
fn strings_comments_and_let_else_are_not_false_positives() -> Result<(), Box<dyn Error>> {
    let parsed = syn::parse_file(
        "fn a() { let Some(value) = Some(1) else { return; };\n\nprintln!(\"pub(crate) else rx\"); }",
    )?;

    assert!(check(&parsed).is_empty());

    Ok(())
}

#[test]
fn cyclic_type_dependencies_are_rejected() -> Result<(), Box<dyn Error>> {
    let parsed = syn::parse_file(
        "struct A; impl A { fn read() -> B { B } } struct B; impl B { fn read() -> A { A } }",
    )?;

    assert!(
        check(&parsed)
            .iter()
            .any(|error| error.contains("dependency B"))
    );

    Ok(())
}
