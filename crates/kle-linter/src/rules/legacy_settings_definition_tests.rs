use super::Visitor;
use crate::diagnostics::Violation;
use std::path::PathBuf;
use syn::visit::Visit;

const SETTINGS_SOURCE: &str = r#"
    struct r#EditorSettings;
    struct AutosavePolicy;
    struct ShortcutMap;
    struct ShortcutBinding;
    struct KeyBinding;
    struct KeyModifier;
    struct SemanticAction;
    fn nested_enums() {
        enum r#EditorSettings { Legacy }
        enum AutosavePolicy { Legacy }
        enum ShortcutMap { Legacy }
        enum ShortcutBinding { Legacy }
        enum KeyBinding { Legacy }
        enum KeyModifier { Legacy }
        enum SemanticAction { Legacy }
    }
    fn nested_aliases() {
        type r#EditorSettings = u8;
        type AutosavePolicy = u16;
        type ShortcutMap = u32;
        type ShortcutBinding = u64;
        type KeyBinding = usize;
        type KeyModifier = isize;
        type SemanticAction = ();
    }
"#;

const UNRELATED_SOURCE: &str = r#"
    struct OtherEditorSettings {}
    enum OtherAutosavePolicy { Enabled }
    type OtherShortcutMap = u8;
    struct OtherShortcutBinding;
    enum OtherKeyBinding { Enter }
    type OtherKeyModifier = u16;
    struct OtherSemanticAction;
    fn sample() {
        let _ = "EditorSettings AutosavePolicy ShortcutMap ShortcutBinding KeyBinding KeyModifier SemanticAction";
    }
"#;

fn lint(source: &str) -> Result<Vec<Violation>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = Visitor::new(PathBuf::from("sample.rs"));
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}

#[test]
fn rejects_all_legacy_settings_names() -> Result<(), syn::Error> {
    let violations = lint(SETTINGS_SOURCE)?;
    assert_eq!(violations.len(), 21);
    assert!(
        violations
            .iter()
            .all(|violation| violation.rule == "legacy-settings-definition")
    );
    for name in [
        "EditorSettings",
        "AutosavePolicy",
        "ShortcutMap",
        "ShortcutBinding",
        "KeyBinding",
        "KeyModifier",
        "SemanticAction",
    ] {
        assert!(
            violations
                .iter()
                .any(|violation| violation.message.contains(name))
        );
    }
    Ok(())
}

#[test]
fn allows_unrelated_settings_names_and_string_literals() -> Result<(), syn::Error> {
    assert!(lint(UNRELATED_SOURCE)?.is_empty());
    Ok(())
}
