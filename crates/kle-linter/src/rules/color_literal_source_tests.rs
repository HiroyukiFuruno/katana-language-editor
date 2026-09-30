use super::ColorLiteralSource;
use crate::diagnostics::KleLintError;
use std::path::PathBuf;

fn source(value: &str) -> ColorLiteralSource<'_> {
    ColorLiteralSource::new(PathBuf::from("sample.rs"), value)
}

#[test]
fn rejects_zero_line_and_column_overflow() {
    assert!(matches!(
        source("abc").offset(0, 0),
        Err(KleLintError::DiagnosticSpan {
            line: 0,
            column: 1,
            ..
        })
    ));
    assert!(matches!(
        source("abc").offset(1, 4),
        Err(KleLintError::DiagnosticSpan {
            line: 1,
            column: 5,
            ..
        })
    ));
}

#[test]
fn excludes_crlf_terminators_from_line_columns() -> Result<(), KleLintError> {
    let source = source("abc\r\ndef");
    assert_eq!(source.offset(1, 3)?, 3);
    assert_eq!(source.offset(2, 0)?, 5);
    assert!(matches!(
        source.offset(1, 4),
        Err(KleLintError::DiagnosticSpan { .. })
    ));
    Ok(())
}

#[test]
fn rejects_empty_spans() {
    let span = proc_macro2::Span::call_site();
    assert_eq!(span.start(), span.end());
    assert!(matches!(
        source("").capture(span),
        Err(KleLintError::DiagnosticSpan { .. })
    ));
}
