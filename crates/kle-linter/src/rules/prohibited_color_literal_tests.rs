use super::ColorLiteralVisitor;
use crate::diagnostics::{KleLintError, Violation};
use std::path::PathBuf;
use syn::visit::Visit;

fn lint(source: &str) -> Result<Vec<Violation>, Box<dyn std::error::Error>> {
    let syntax = syn::parse_file(source)?;
    let mut visitor = ColorLiteralVisitor::new(PathBuf::from("sample.rs"), source);
    visitor.visit_file(&syntax);
    Ok(visitor.finish()?)
}

#[test]
fn detects_color_constants_and_literals() -> Result<(), Box<dyn std::error::Error>> {
    let violations = lint(
        r##"
        fn sample() {
            let _ = egui::Color32::RED;
            let _ = egui::Color32::from_rgb(1, 2, 3);
            let _ = "#fff";
            let _ = "rgba(1, 2, 3, 0.5)";
        }
        "##,
    )?;
    assert_eq!(violations.len(), 4);
    assert!(violations.iter().all(|violation| {
        violation
            .message
            .starts_with("KUC owns host presentation color resolution;")
    }));
    assert_eq!(violations[0].literal.as_deref(), Some("egui::Color32::RED"));
    assert_eq!(
        violations[1].literal.as_deref(),
        Some("egui::Color32::from_rgb(1, 2, 3)")
    );
    assert_eq!(violations[2].literal.as_deref(), Some("\"#fff\""));
    assert!(violations.iter().all(|violation| violation.hint.is_some()));
    Ok(())
}

#[test]
fn nonliteral_arguments_are_not_color_literals() -> Result<(), Box<dyn std::error::Error>> {
    let violations = lint(
        r#"
        fn sample(color: Rgba) {
            let _ = egui::Color32::from_rgba_premultiplied(
                color.red,
                color.green,
                color.blue,
                color.alpha,
            );
        }
        "#,
    )?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn explicit_cfg_test_module_is_not_production() -> Result<(), Box<dyn std::error::Error>> {
    let violations =
        lint("#[cfg(test)] mod tests { fn sample() { let _ = egui::Color32::RED; } }")?;
    assert!(violations.is_empty());
    Ok(())
}

#[test]
fn captures_unicode_crlf_raw_and_multiline_source() -> Result<(), Box<dyn std::error::Error>> {
    let violations = lint(
        "fn sample() {\r\n    let \u{65e5}\u{672c}\u{8a9e} = \"\u{2b50}\u{fe0f}\"; let _ = egui::Color32::RED;\r\n    let _ = r##\"#ff00aa\"##;\r\n    let _ = egui::Color32::from_rgb(\r\n        1, 2, 3,\r\n    );\r\n}",
    )?;
    assert_eq!(violations.len(), 3);
    assert_eq!(violations[0].literal.as_deref(), Some("egui::Color32::RED"));
    assert_eq!((violations[0].line, violations[0].column), (2, 29));
    assert_eq!(violations[1].literal.as_deref(), Some("r##\"#ff00aa\"##"));
    assert_eq!(
        violations[2].literal.as_deref(),
        Some("egui::Color32::from_rgb(\r\n        1, 2, 3,\r\n    )")
    );
    Ok(())
}

#[test]
fn captures_hsl_and_hsla_raw_literals() -> Result<(), Box<dyn std::error::Error>> {
    let violations = lint(
        "fn sample() { let _ = r##\"HSL(120, 100%, 50%)\"##; let _ = r#\"hsla(10, 20%, 30%, .4)\"#; }",
    )?;
    assert_eq!(violations.len(), 2);
    assert_eq!(
        violations[0].literal.as_deref(),
        Some("r##\"HSL(120, 100%, 50%)\"##")
    );
    assert_eq!(
        violations[1].literal.as_deref(),
        Some("r#\"hsla(10, 20%, 30%, .4)\"#")
    );
    assert_eq!(
        violations[0].hint.as_deref(),
        Some("Remove the KLE color literal and use the KUC-owned presentation color.")
    );
    assert_eq!(violations[1].hint, violations[0].hint);
    Ok(())
}

#[test]
fn rejects_out_of_range_span() -> Result<(), Box<dyn std::error::Error>> {
    let syntax = syn::parse_file("fn sample() { let _ = egui::Color32::RED; }")?;
    let mut visitor = ColorLiteralVisitor::new(PathBuf::from("sample.rs"), "fn sample() {}");
    visitor.visit_file(&syntax);
    assert!(matches!(
        visitor.finish(),
        Err(KleLintError::DiagnosticSpan { .. })
    ));
    Ok(())
}

#[test]
fn discards_prior_violations_when_a_later_span_is_out_of_range()
-> Result<(), Box<dyn std::error::Error>> {
    let syntax =
        syn::parse_file("fn sample() { let _ = egui::Color32::RED; let _ = egui::Color32::RED; }")?;
    let mut visitor = ColorLiteralVisitor::new(
        PathBuf::from("sample.rs"),
        "fn sample() { let _ = egui::Color32::RED; ",
    );
    visitor.visit_file(&syntax);
    assert!(matches!(
        visitor.finish(),
        Err(KleLintError::DiagnosticSpan { .. })
    ));
    Ok(())
}
