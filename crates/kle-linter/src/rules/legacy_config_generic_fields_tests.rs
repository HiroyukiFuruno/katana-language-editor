use super::{Visitor, is_neutral_source};
use crate::diagnostics::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

fn lint(source: &str) -> Result<Vec<Violation>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = Visitor::new(PathBuf::from("sample.rs"));
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}

#[test]
fn detects_all_forbidden_fields_in_both_config_structs() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct EditorConfig {
            pub typography: u8,
            pub spacing: u8,
            pub settings: u8,
            pub theme: u8,
            pub strings: u8,
            pub locale: u8,
        }
        struct EditorConfigInput {
            pub typography: u8,
            pub spacing: u8,
            pub settings: u8,
            pub theme: u8,
            pub strings: u8,
            pub locale: u8,
        }
        "#,
    )?;
    assert_eq!(violations.len(), 12);
    Ok(())
}

#[test]
fn detects_private_fields() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct EditorConfig {
            typography: u8,
            spacing: u8,
            settings: u8,
            theme: u8,
            strings: u8,
            locale: u8,
        }
        "#,
    )?;
    assert_eq!(violations.len(), 6);
    Ok(())
}

#[test]
fn allows_unrelated_structs_and_literals() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct OtherConfig {
            typography: u8,
            spacing: u8,
            settings: u8,
            theme: u8,
            strings: u8,
            locale: u8,
        }
        fn sample() {
            let _ = "EditorConfig typography spacing settings theme strings locale";
        }
        "#,
    )?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn rejects_raw_identifiers() -> Result<(), syn::Error> {
    let violations = lint("struct r#EditorConfig { r#theme: u8, r#strings: u8, r#locale: u8 }")?;
    assert_eq!(violations.len(), 3);
    Ok(())
}

#[test]
fn detects_nested_config_structs_without_nested_unrelated_fields() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        fn nested() {
            struct EditorConfig { theme: u8, strings: u8, locale: u8 }
            struct r#EditorConfigInput { r#theme: u8, r#strings: u8, r#locale: u8 }
            const _: () = { struct Other { theme: u8, strings: u8, locale: u8 } };
        }
        "#,
    )?;
    assert_eq!(violations.len(), 6);
    Ok(())
}

#[test]
fn rejects_style_names_across_definition_kinds_and_nesting() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct r#Typography {}
        enum Spacing { Compact }
        type Typography = u8;
        fn nested() {
            struct r#Spacing;
            enum r#Typography { Expanded }
            type Spacing = u16;
        }
        "#,
    )?;
    assert_eq!(violations.len(), 6);
    assert!(
        violations
            .iter()
            .all(|violation| violation.rule == "legacy-ui-style-definition")
    );
    Ok(())
}

#[test]
fn allows_unrelated_style_names_and_string_literals() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct OtherTypography {}
        enum OtherSpacing { Compact }
        type OtherTypographyAlias = Typography;
        fn sample() {
            let _ = "Typography Spacing";
        }
        "#,
    )?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn parse_failures_propagate() {
    assert!(lint("struct Typography {").is_err());
}

#[test]
fn allows_unrelated_struct_inside_const_field_type() -> Result<(), syn::Error> {
    let violations = lint(
        "struct EditorConfig { bytes: [u8; { struct Other { theme: u8, strings: u8, locale: u8 } 1 }] }",
    )?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn selects_only_neutral_crate_sources() {
    let root = Path::new("workspace");
    assert!(is_neutral_source(
        root,
        &root.join("crates/katana-language-editor/src/config.rs")
    ));
    assert!(is_neutral_source(
        root,
        &root.join("crates/katana-language-editor/src/rules/config_tests.rs")
    ));
    for path in [
        "crates/katana-language-editor-egui/src/config.rs",
        "crates/katana-language-editor/src-other/config.rs",
        "tools/kle-storybook/src/config.rs",
    ] {
        assert!(!is_neutral_source(root, &root.join(path)));
    }
}
