use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;
#[path = "kuc_text_runtime_command_chrome_event_mapping_support.rs"]
mod support;
use support::{is_kuc_scroll_delegate, is_target};

const FORBIDDEN: &[&str] = &["Area", "TextEdit", "Button", "EditorPixel", "Fallback"];
const SOURCE_METHODS: &[&str] = &[
    "lines",
    "split",
    "split_terminator",
    "chars",
    "char_indices",
    "line_for_offset",
    "scroll_to_line",
    "scroll_by_pixels",
];
const FORBIDDEN_MESSAGE: &str = "KUC command chrome mapping must not create controls, pixel DTOs, fallbacks, or local source scroll logic.";
const STRING_SWITCH_MESSAGE: &str = "KUC command chrome mapping must resolve opaque ids through the injected typed target map, not a string match.";
const FORM_STATE_MESSAGE: &str =
    "KUC command chrome mapping must not persist a local query or replacement form.";
const CURRENT_OUTPUT_MESSAGE: &str =
    "widget.rs must map the current KUC command chrome output exactly once.";
const KUC_SCROLL_MESSAGE: &str =
    "active search scrolling must delegate to EditorScrollControl::scroll_into_view.";

pub(super) struct CommandChromeEventMappingRule;

impl CommandChromeEventMappingRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace
            .rust_files()
            .iter()
            .filter(|file| is_target(file.path()))
        {
            let mut visitor = CommandChromeEventMappingVisitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

struct CommandChromeEventMappingVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    current_output_maps: usize,
    in_active_search_scroll: bool,
    kuc_scroll_delegate: bool,
}

impl CommandChromeEventMappingVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            current_output_maps: 0,
            in_active_search_scroll: false,
            kuc_scroll_delegate: false,
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn push(&mut self, span: proc_macro2::Span, message: &'static str) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            "kuc-text-runtime",
            message,
        ));
    }

    fn file_is(&self, value: &str) -> bool {
        self.file.file_name().is_some_and(|name| name == value)
    }

    fn rejects_source_method(&self) -> bool {
        !self.file_is("search_control.rs") || self.in_active_search_scroll
    }

    fn has_forbidden_path(node: &syn::Path) -> bool {
        node.segments.iter().any(|segment| {
            FORBIDDEN
                .iter()
                .any(|value| segment.ident.to_string().contains(value))
        })
    }

    fn string_match(value: &syn::Expr) -> bool {
        matches!(
            value,
            syn::Expr::Lit(syn::ExprLit {
                lit: syn::Lit::Str(_),
                ..
            })
        ) || matches!(value, syn::Expr::MethodCall(call) if call.method == "as_str")
    }
}

impl<'ast> Visit<'ast> for CommandChromeEventMappingVisitor {
    fn visit_file(&mut self, node: &'ast syn::File) {
        syn::visit::visit_file(self, node);
        if self.file_is("widget.rs") && self.current_output_maps != 1 {
            self.push(node.span(), CURRENT_OUTPUT_MESSAGE);
        }
        if self.file_is("search_control.rs") && !self.kuc_scroll_delegate {
            self.push(node.span(), KUC_SCROLL_MESSAGE);
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if Self::has_forbidden_path(node) {
            self.push(node.span(), FORBIDDEN_MESSAGE);
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "map_kuc_command_chrome_output" {
            self.current_output_maps += 1;
        }
        if self.rejects_source_method() && SOURCE_METHODS.iter().any(|value| node.method == *value)
        {
            self.push(node.method.span(), FORBIDDEN_MESSAGE);
        }
        if self.in_active_search_scroll && node.method == "scroll_into_view" {
            self.kuc_scroll_delegate = true;
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_match(&mut self, node: &'ast syn::ExprMatch) {
        if Self::string_match(&node.expr) {
            self.push(node.match_token.span(), STRING_SWITCH_MESSAGE);
        }
        syn::visit::visit_expr_match(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.in_active_search_scroll && is_kuc_scroll_delegate(&node.func) {
            self.kuc_scroll_delegate = true;
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if node.ident.to_string().contains("Form") || node.ident.to_string().ends_with("State") {
            let persists_form_value = node.fields.iter().any(|field| {
                field.ident.as_ref().is_some_and(|ident| {
                    matches!(
                        ident.to_string().as_str(),
                        "query" | "replace" | "replace_value"
                    )
                })
            });
            if persists_form_value {
                self.push(node.ident.span(), FORM_STATE_MESSAGE);
            }
        }
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let previous = self.in_active_search_scroll;
        self.in_active_search_scroll =
            self.file_is("search_control.rs") && node.sig.ident == "scroll_to_active_search_match";
        syn::visit::visit_impl_item_fn(self, node);
        self.in_active_search_scroll = previous;
    }
}

#[cfg(test)]
#[path = "kuc_text_runtime_command_chrome_event_mapping_tests.rs"]
mod tests;
