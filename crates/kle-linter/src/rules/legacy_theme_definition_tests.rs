use super::{Visitor, is_neutral_source};
use crate::diagnostics::Violation;
use std::path::{Path, PathBuf};
use syn::visit::Visit;

const THEME_NAMES: [&str; 5] = [
    "Rgba",
    "ColorTokens",
    "ColorTokensInput",
    "Theme",
    "EditorTheme",
];
const ALL_THEME_DEFINITIONS: &str = r#"
    struct r#Rgba;
    struct ColorTokens;
    struct ColorTokensInput;
    struct Theme;
    struct EditorTheme;
    fn nested() {
        enum Rgba { Legacy }
        enum r#ColorTokens { Legacy }
        enum ColorTokensInput { Legacy }
        enum r#Theme { Legacy }
        enum EditorTheme { Legacy }
    }
    type r#Rgba = u64;
    type ColorTokens = u8;
    type ColorTokensInput = u16;
    type Theme = u32;
    type EditorTheme = u64;
"#;

fn lint(source: &str) -> Result<Vec<Violation>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = Visitor::new(PathBuf::from("sample.rs"));
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}

#[test]
fn rejects_all_theme_names_as_structs_enums_and_type_aliases() -> Result<(), syn::Error> {
    let violations = lint(ALL_THEME_DEFINITIONS)?;
    assert_eq!(violations.len(), 15);
    for name in THEME_NAMES {
        let expected_message =
            format!("{name} must not be retained as a legacy UI style definition.");
        let matches = violations
            .iter()
            .filter(|violation| violation.message == expected_message)
            .count();
        assert_eq!(matches, 3, "expected three diagnostics for {name}");
    }
    assert!(
        violations
            .iter()
            .all(|violation| violation.rule == "legacy-ui-style-definition")
    );
    Ok(())
}

#[test]
fn rejects_private_nested_and_raw_theme_definitions() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct Rgba {}
        enum r#ColorTokens { Legacy }
        type ColorTokensInput = u8;
        fn nested() {
            struct r#Theme;
            enum EditorTheme { Legacy }
        }
        "#,
    )?;
    assert_eq!(violations.len(), 5);
    for name in THEME_NAMES {
        let expected_message =
            format!("{name} must not be retained as a legacy UI style definition.");
        assert!(
            violations
                .iter()
                .any(|violation| violation.message == expected_message)
        );
    }
    Ok(())
}

#[test]
fn allows_unrelated_theme_names_and_literals() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        struct OtherRgba {}
        enum OtherColorTokens { Legacy }
        type OtherColorTokensInput = u8;
        struct OtherTheme;
        enum OtherEditorTheme { Legacy }
        fn sample() {
            let _ = "Rgba ColorTokens ColorTokensInput Theme EditorTheme";
        }
        "#,
    )?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn propagates_parse_errors() {
    assert!(lint("struct EditorTheme {").is_err());
}

#[test]
fn selects_only_neutral_crate_sources() {
    let root = Path::new("workspace");
    assert!(is_neutral_source(
        root,
        &root.join("crates/katana-language-editor/src/theme.rs")
    ));
    assert!(!is_neutral_source(
        root,
        &root.join("crates/katana-language-editor-egui/src/theme.rs")
    ));
}
