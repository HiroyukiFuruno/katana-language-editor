use super::ProhibitedAttributeVisitor;
use crate::diagnostics::Violation;
use std::path::PathBuf;
use syn::visit::Visit;

fn lint(source: &str) -> Result<Vec<Violation>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = ProhibitedAttributeVisitor::new(PathBuf::from("fixture.rs"));
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}

#[test]
fn rejects_inner_allow_attribute() -> Result<(), syn::Error> {
    assert_eq!(lint("#![allow(dead_code)] fn fixture() {}")?.len(), 1);
    Ok(())
}

#[test]
fn rejects_outer_allow_attribute() -> Result<(), syn::Error> {
    assert_eq!(lint("#[allow(dead_code)] fn fixture() {}")?.len(), 1);
    Ok(())
}

#[test]
fn rejects_cfg_attr_allow_attribute() -> Result<(), syn::Error> {
    assert_eq!(
        lint("#[cfg_attr(test, allow(dead_code))] fn fixture() {}")?.len(),
        1
    );
    Ok(())
}

#[test]
fn accepts_ordinary_attributes() -> Result<(), syn::Error> {
    assert!(lint("#[derive(Debug)] struct Fixture;")?.is_empty());
    Ok(())
}

#[test]
fn color_allow_with_unrelated_token_is_not_exempt() -> Result<(), syn::Error> {
    let violations = lint("#[allow(prohibited_color_literal, dead_code)] fn fixture() {}")?;
    assert_eq!(violations.len(), 1);
    Ok(())
}

#[test]
fn rejects_qualified_color_allow_on_file_and_item() -> Result<(), syn::Error> {
    let violations = lint(
        "#![allow(kle_lint::prohibited_color_literal)]\n#[allow(kle_lint::prohibited_color_literal)] fn fixture() {}",
    )?;
    assert_eq!(violations.len(), 2);
    assert!(
        violations
            .iter()
            .all(|violation| violation.rule == "prohibited-attribute")
    );
    Ok(())
}
