use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;

const FORBIDDEN_PATHS: &[&str] = &[
    "EguiContextMenuAdapter",
    "ContextMenuAnchor",
    "ContextMenuPlacement",
    "UiContextMenuAnchor",
    "UiContextMenuRect",
    "Area",
    "ContextMenuCompositor",
    "ArtifactPaintPlanRef",
];
const FORBIDDEN_METHODS: &[&str] = &[
    "show_context_menu",
    "set_context_menu",
    "synchronize_context_menu",
    "artifact_paint_plans",
];
const MESSAGE: &str = "KLE context menu must use the retained KUC text-command root and may not own geometry, adapter, or artifact composition.";
const FILES: &[&str] = &[
    "kuc_context_menu_host_presentation.rs",
    "kuc_context_menu_host_validation.rs",
    "kuc_context_menu_binding.rs",
    "kuc_context_menu_composition.rs",
    "kuc_text_surface_binding.rs",
    "kuc_artifact_aggregate.rs",
    "widget.rs",
    "widget_state.rs",
];

pub(super) struct ContextMenuRuntimeRule;

impl ContextMenuRuntimeRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        Ok(workspace
            .rust_files()
            .iter()
            .filter(|file| {
                file.path()
                    .file_name()
                    .is_some_and(|name| FILES.iter().any(|candidate| name == *candidate))
            })
            .flat_map(|file| {
                let mut visitor = Visitor::new(file.path().to_path_buf());
                visitor.visit_file(file.syntax());
                visitor.violations
            })
            .collect())
    }
}

struct Visitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_target_builder: bool,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_target_builder: false,
        }
    }
    fn push(&mut self, span: proc_macro2::Span) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            "kuc-text-runtime-context-menu",
            MESSAGE,
        ));
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if FORBIDDEN_PATHS
            .iter()
            .any(|name| node.segments.iter().any(|segment| segment.ident == *name))
            || (self.in_target_builder
                && node
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "IngestClipboardFileUrls"))
        {
            self.push(node.span());
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if FORBIDDEN_METHODS.iter().any(|name| node.method == *name) {
            self.push(node.method.span());
        }
        let previous = self.in_target_builder;
        self.in_target_builder = node.method == "with_target";
        syn::visit::visit_expr_method_call(self, node);
        self.in_target_builder = previous;
    }
}

#[cfg(test)]
#[path = "kuc_text_runtime_context_menu_tests.rs"]
mod tests;
