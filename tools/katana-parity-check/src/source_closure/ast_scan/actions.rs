use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{ItemEnum, Path as SynPath};

use super::super::ast_resolution::span_to_text;
use super::SourceClosureVisitor;

impl<'a> SourceClosureVisitor<'a> {
    pub(crate) fn record_app_action_definition(&mut self, node: &ItemEnum) {
        if node.ident != "AppAction" {
            return;
        }
        let enum_span = span_to_text(self.current_relative_path, &node.span());
        for variant in &node.variants {
            self.state.record_action_definition(
                self.current_relative_path,
                &self.current_symbol(),
                enum_span.clone(),
                variant.ident.to_string(),
                span_to_text(self.current_relative_path, &variant.span()),
            );
        }
    }

    pub(crate) fn record_app_action_construction(
        &mut self,
        path: &SynPath,
        span: Span,
        style: &str,
    ) {
        let segments = path
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect::<Vec<_>>();
        let Some(first) = segments.first() else {
            return;
        };
        let (enum_name, variant, unresolved_reason) = if first == "AppAction" {
            match segments.as_slice() {
                [_, variant] => ("AppAction".to_string(), Some(variant.clone()), None),
                _ => (
                    "AppAction".to_string(),
                    segments.get(1).cloned(),
                    Some(
                        "direct AppAction construction path does not name exactly one variant"
                            .to_string(),
                    ),
                ),
            }
        } else if self.has_import_alias(first) {
            (
                first.clone(),
                segments.get(1).cloned(),
                Some(format!(
                    "imported AppAction-like construction path `{}` is not proven to resolve to an indexed AppAction definition",
                    segments.join("::")
                )),
            )
        } else if let [_, .., enum_segment, variant] = segments.as_slice()
            && enum_segment == "AppAction"
        {
            (
                "AppAction".to_string(),
                Some(variant.clone()),
                Some(format!(
                    "qualified AppAction construction path `{}` is not proven to resolve to an indexed AppAction definition",
                    segments.join("::")
                )),
            )
        } else {
            return;
        };
        self.state
            .record_action_construction(super::super::scan_state::ActionConstruction {
                file: self.current_relative_path.to_string(),
                symbol: self.current_symbol(),
                span: span_to_text(self.current_relative_path, &span),
                enum_name,
                variant,
                style: style.to_string(),
                unresolved_reason,
                input_origin_candidates: self.input_origin_candidates(span),
            });
    }

    pub(crate) fn record_app_action_macro(&mut self, path: &SynPath, span: Span) {
        let before = self.state.action_constructions.len();
        self.record_app_action_construction(path, span, "macro");
        if let Some(construction) = self.state.action_constructions.get_mut(before) {
            let macro_reason = "macro construction cannot be proven from the unexpanded syn AST";
            construction.unresolved_reason = Some(
                construction
                    .unresolved_reason
                    .as_deref()
                    .filter(|reason| reason.starts_with("qualified AppAction construction path `"))
                    .map_or_else(
                        || macro_reason.to_string(),
                        |reason| format!("{reason}; {macro_reason}"),
                    ),
            );
        }
    }
}
