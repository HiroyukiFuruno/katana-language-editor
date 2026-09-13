use syn::visit::{self, Visit};
use syn::{Expr, ExprCall, ExprMacro, ExprMethodCall, spanned::Spanned};

use super::syntax::{expr_to_syntax, path_to_syntax};

pub(super) fn input_candidates_for_condition(
    condition: &Expr,
    file: &str,
    fallback_span: proc_macro2::Span,
) -> Vec<super::scan_state::InputOriginCandidate> {
    let mut visitor = ConditionCandidateVisitor {
        file: file.to_string(),
        candidates: Vec::new(),
    };
    visitor.visit_expr(condition);
    if visitor.candidates.is_empty() {
        visitor
            .candidates
            .push(super::scan_state::InputOriginCandidate {
                kind: "unknown_input_condition".to_string(),
                condition_syntax: expr_to_syntax(condition),
                condition_span: super::ast_resolution::span_to_text(file, &fallback_span),
                boundary_syntax: "if_condition".to_string(),
                boundary_span: super::ast_resolution::span_to_text(file, &fallback_span),
                unresolved_reason: Some(
                    "if condition is not a structurally recognized input candidate".to_string(),
                ),
            });
    }
    visitor.candidates
}

struct ConditionCandidateVisitor {
    file: String,
    candidates: Vec<super::scan_state::InputOriginCandidate>,
}

impl ConditionCandidateVisitor {
    fn candidate(
        &mut self,
        kind: &str,
        condition_syntax: String,
        boundary_syntax: String,
        span: proc_macro2::Span,
        unresolved_reason: Option<&str>,
    ) {
        self.candidates
            .push(super::scan_state::InputOriginCandidate {
                kind: kind.to_string(),
                condition_syntax,
                condition_span: super::ast_resolution::span_to_text(&self.file, &span),
                boundary_syntax,
                boundary_span: super::ast_resolution::span_to_text(&self.file, &span),
                unresolved_reason: unresolved_reason.map(str::to_string),
            });
    }
}

impl<'ast> Visit<'ast> for ConditionCandidateVisitor {
    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        let method = node.method.to_string();
        let kind = if egui_activation_method(&method) {
            Some("egui_response_activation_condition")
        } else if keyboard_consumption_method(&method) {
            Some("keyboard_shortcut_consumption_condition")
        } else if accesskit_action_method(&method) {
            Some("accesskit_action_condition")
        } else {
            None
        };
        if let Some(kind) = kind {
            self.candidate(
                kind,
                format!("{}.{}", expr_to_syntax(&node.receiver), method),
                method,
                node.span(),
                None,
            );
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = node.func.as_ref() {
            let syntax = path_to_syntax(&path.path);
            let lowered = syntax.to_ascii_lowercase();
            let kind = if lowered.contains("shortcut") {
                Some("keyboard_shortcut_consumption_condition")
            } else if lowered.contains("accesskit") || lowered.contains("accessibility_action") {
                Some("accesskit_action_condition")
            } else {
                None
            };
            if let Some(kind) = kind {
                self.candidate(kind, syntax.clone(), syntax, node.span(), None);
            }
        }
        visit::visit_expr_call(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast ExprMacro) {
        self.candidate(
            "unknown_input_condition",
            path_to_syntax(&node.mac.path),
            path_to_syntax(&node.mac.path),
            node.span(),
            Some("macro-generated input condition cannot be proven from syn AST"),
        );
    }
}

fn egui_activation_method(method: &str) -> bool {
    matches!(
        method,
        "clicked"
            | "double_clicked"
            | "secondary_clicked"
            | "middle_clicked"
            | "clicked_by"
            | "drag_started"
            | "dragged"
            | "drag_stopped"
            | "changed"
    )
}

fn keyboard_consumption_method(method: &str) -> bool {
    method == "consume_shortcut" || method == "shortcut_pressed" || method == "key_pressed"
}

fn accesskit_action_method(method: &str) -> bool {
    method.contains("accesskit")
        || method.contains("accessibility_action")
        || method.contains("action_requested")
        || method.contains("requested_action")
}
