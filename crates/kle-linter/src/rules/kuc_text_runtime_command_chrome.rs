use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;

const FORBIDDEN: &[&str] = &[
    "Area",
    "TextEdit",
    "Button",
    "EditorPixel",
    "Fallback",
    "UiRect",
    "Pos2",
    "RawInput",
];
const FORBIDDEN_METHODS: &[&str] = &[
    "show_kuc_command_chrome",
    "run_ui",
    "pointer_button",
    "pointer_click",
    "click_position",
];
const FORBIDDEN_MESSAGE: &str =
    "KUC command chrome composition must not create egui controls, pixel DTOs, or fallbacks.";

pub(super) struct CommandChromeRuntimeRule;

impl CommandChromeRuntimeRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files().iter().filter(|file| {
            file.path()
                .file_name()
                .is_some_and(|name| name == "kuc_command_chrome_composition.rs")
        }) {
            let mut visitor = CommandChromeRuntimeVisitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

struct CommandChromeRuntimeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl CommandChromeRuntimeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn push_violation(&mut self, span: proc_macro2::Span, message: &'static str) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            "kuc-text-runtime",
            message,
        ));
    }

    fn has_forbidden_path(node: &syn::Path) -> bool {
        node.segments.iter().any(|segment| {
            FORBIDDEN
                .iter()
                .any(|value| segment.ident.to_string().contains(value))
        })
    }
}

impl<'ast> Visit<'ast> for CommandChromeRuntimeVisitor {
    fn visit_file(&mut self, node: &'ast syn::File) {
        syn::visit::visit_file(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if Self::has_forbidden_path(node) {
            self.push_violation(node.span(), FORBIDDEN_MESSAGE);
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "button"
            || node.method.to_string().contains("fallback")
            || FORBIDDEN_METHODS
                .iter()
                .any(|method| node.method == *method)
        {
            self.push_violation(node.method.span(), FORBIDDEN_MESSAGE);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
#[path = "kuc_text_runtime_command_chrome_tests.rs"]
mod tests;
