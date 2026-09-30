use super::{KleLintError, Violation, ViolationReport};
use std::path::PathBuf;

#[test]
fn legacy_violation_report_is_exact() {
    let violation = Violation::new(PathBuf::from("src/lib.rs"), 4, 2, "rule", "message");

    assert_eq!(
        ViolationReport::format(&[violation]),
        "\n[AST lint]\nsrc/lib.rs:4:2 [rule] message\n"
    );
}

#[test]
fn new_violation_has_no_literal_or_hint() {
    let violation = Violation::new(PathBuf::from("src/lib.rs"), 1, 1, "rule", "message");

    assert_eq!(violation.literal, None);
    assert_eq!(violation.hint, None);
}

#[test]
fn literal_hint_builder_populates_structured_fields_and_report() {
    let violation = Violation::new(PathBuf::from("src/lib.rs"), 4, 2, "rule", "message")
        .with_literal_hint(
            "egui::Color32::RED",
            "resolve this through host presentation",
        );

    assert_eq!(violation.literal.as_deref(), Some("egui::Color32::RED"));
    assert_eq!(
        violation.hint.as_deref(),
        Some("resolve this through host presentation")
    );
    assert_eq!(
        ViolationReport::format(&[violation]),
        "\n[AST lint]\nsrc/lib.rs:4:2 [rule] message\n  literal: \"egui::Color32::RED\"\n  hint: \"resolve this through host presentation\"\n"
    );
}

#[test]
fn literal_hint_report_escapes_unicode_controls_and_multiline_values() {
    let violation = Violation::new(PathBuf::from("src/lib.rs"), 1, 1, "rule", "message")
        .with_literal_hint("色\n\t\u{0000}\r\u{001b}🙂", "hint\nnext");
    let report = ViolationReport::format(&[violation]);

    assert_eq!(
        report,
        "\n[AST lint]\nsrc/lib.rs:1:1 [rule] message\n  literal: \"色\\n\\t\\0\\r\\u{1b}🙂\"\n  hint: \"hint\\nnext\"\n"
    );
    assert_eq!(report.matches("[rule]").count(), 1);
    assert_eq!(report.lines().count(), 5);
}

#[test]
fn diagnostic_span_error_preserves_location() {
    let error = KleLintError::DiagnosticSpan {
        path: PathBuf::from("src/lib.rs"),
        line: 7,
        column: 9,
    };

    assert_eq!(
        error.to_string(),
        "failed to resolve diagnostic span in src/lib.rs:7:9"
    );
}
