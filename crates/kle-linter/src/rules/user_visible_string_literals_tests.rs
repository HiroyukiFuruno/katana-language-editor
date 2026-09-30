use super::UserVisibleStringVisitor;
use crate::diagnostics::Violation;
use std::path::PathBuf;
use syn::visit::Visit;

fn lint(source: &str) -> Result<Vec<Violation>, syn::Error> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = UserVisibleStringVisitor::new(PathBuf::from("sample.rs"));
    visitor.visit_file(&syntax);
    Ok(visitor.into_violations())
}

#[test]
fn detects_visible_strings() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        fn sample(ui: &mut egui::Ui) {
            ui.label("Copy");
            let _ = egui::Button::new("Save");
        }
        "#,
    )?;
    assert_eq!(violations.len(), 2);
    Ok(())
}

#[test]
fn detects_tooltip_and_macro_wrapped_visible_strings() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        fn sample(ui: &mut egui::Ui, name: &str) {
            ui.button(strings.save()).on_hover_text("Save the document");
            ui.label(format!("Open {name}"));
            let _ = egui::RichText::new(concat!("Close", " document"));
        }
        "#,
    )?;
    assert_eq!(violations.len(), 3);
    Ok(())
}

#[test]
fn detects_nested_and_extended_visible_strings() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        fn sample(ui: &mut egui::Ui, dirty: bool) {
            ui.strong(if dirty { "Unsaved" } else { "Saved" });
            ui.colored_label(egui::Color32::RED, match dirty { true => "Error", false => "Ready" });
            ui.hyperlink_to("Open", "https://example.invalid");
        }
        "#,
    )?;
    assert_eq!(violations.len(), 3);
    Ok(())
}

#[test]
fn ignores_internal_error_strings() -> Result<(), syn::Error> {
    let violations = lint(
        r#"
        fn sample() -> EditorError {
            EditorError::Unsupported("not visible".to_string())
        }
        "#,
    )?;
    assert!(violations.is_empty());
    Ok(())
}
