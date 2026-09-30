use super::super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;

pub(super) fn create(
    path: &str,
    target: String,
    span: proc_macro2::Span,
) -> ExternalUiDirectSemanticEdge {
    ExternalUiDirectSemanticEdge {
        from_symbol: "TextEdit::show".into(),
        target_symbol: target,
        kind: "closure_callback_call".into(),
        source_file: format!("egui/{path}"),
        span: format!(
            "egui:{path}:{}:{}-{}:{}",
            span.start().line,
            span.start().column,
            span.end().line,
            span.end().column
        ),
    }
}
