use super::architecture::{EGUI_CRATE, LIB_CRATE};
use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;
#[path = "kuc_text_runtime_command_chrome.rs"]
mod command_chrome;
use command_chrome::CommandChromeRuntimeRule;
#[path = "kuc_text_runtime_command_chrome_event_mapping.rs"]
mod command_chrome_event_mapping;
use command_chrome_event_mapping::CommandChromeEventMappingRule;
const DIRECT_SHOW_FORBIDDEN: &[&str] = &["PlatformTextSurface", "ScrollArea", "TextEdit", "Frame"];
const DIRECT_SHOW_MESSAGE: &str = "EguiLanguageEditor::show must invoke the KUC TextSurface binding directly; legacy renderer and generic egui surface types are forbidden in this path.";
const DIRECT_KUC_SHOW_COUNT_MESSAGE: &str =
    "public EguiLanguageEditor::show must directly invoke self.kuc_text_surface.show exactly once.";
const SHOW_WITH_CHROME_MESSAGE: &str =
    "public EguiLanguageEditor::show must not invoke self.kuc_text_surface.show_with_chrome.";
const GUTTER_FORBIDDEN_METHODS: &[&str] = &[
    "lines",
    "split",
    "split_terminator",
    "chars",
    "char_indices",
];
const GUTTER_LOCAL_MODEL_MESSAGE: &str =
    "gutter_control.rs must not create or consume a local gutter model; use KUC frame facts.";
const GUTTER_SOURCE_ENUMERATION_MESSAGE: &str =
    "gutter_control.rs must not enumerate source text to reconstruct gutter rows.";
const MISSING_GUTTER_FRAME_MESSAGE: &str =
    "gutter_control.rs must consume the latest KUC frame before exposing gutter facts.";
const LEGACY_COMMAND_CHROME_SHOW_MESSAGE: &str = "EguiLanguageEditor::show must compose command chrome through the KUC root surface, not show it separately.";
pub struct KucTextRuntimeRule;
impl KucTextRuntimeRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        let core_root = workspace.root().join(LIB_CRATE).join("src");
        let egui_root = workspace.root().join(EGUI_CRATE).join("src");
        for file in workspace.rust_files() {
            if !(file.is_under(&core_root) || file.is_under(&egui_root)) {
                continue;
            }
            let mut visitor = KucTextRuntimeVisitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        violations.extend(CommandChromeRuntimeRule::check(workspace)?);
        violations.extend(CommandChromeEventMappingRule::check(workspace)?);
        Ok(violations)
    }
}
struct KucTextRuntimeVisitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_direct_show: bool,
    direct_binding_show_count: usize,
    consumes_latest_kuc_gutter_frame: bool,
}
impl KucTextRuntimeVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_direct_show: false,
            direct_binding_show_count: 0,
            consumes_latest_kuc_gutter_frame: false,
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
    fn is_file(&self, expected: &str) -> bool {
        self.file.file_name().is_some_and(|name| name == expected)
    }
    fn is_local_gutter_model(path: &syn::Path) -> bool {
        path.segments.iter().any(|segment| {
            let name = segment.ident.to_string();
            name == "LineGutterModel" || name.ends_with("GutterModel")
        })
    }
}
impl<'ast> Visit<'ast> for KucTextRuntimeVisitor {
    fn visit_file(&mut self, node: &'ast syn::File) {
        syn::visit::visit_file(self, node);
        if self.is_file("gutter_control.rs") && !self.consumes_latest_kuc_gutter_frame {
            self.push_violation(node.span(), MISSING_GUTTER_FRAME_MESSAGE);
        }
    }
    fn visit_path(&mut self, node: &'ast syn::Path) {
        if self.in_direct_show
            && node.segments.iter().any(|segment| {
                DIRECT_SHOW_FORBIDDEN
                    .iter()
                    .any(|value| segment.ident == *value)
            })
        {
            self.push_violation(node.span(), DIRECT_SHOW_MESSAGE);
        }
        if self.is_file("gutter_control.rs") && Self::is_local_gutter_model(node) {
            self.push_violation(node.span(), GUTTER_LOCAL_MODEL_MESSAGE);
        }
        syn::visit::visit_path(self, node);
    }
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_direct_show
            && matches!(&*node.receiver, syn::Expr::Field(field)
                if matches!(&field.member, syn::Member::Named(member) if member == "kuc_text_surface"))
        {
            if node.method == "show" {
                self.direct_binding_show_count += 1;
            }
            if node.method == "show_with_chrome" {
                self.push_violation(node.method.span(), SHOW_WITH_CHROME_MESSAGE);
            }
        }
        if self.is_file("gutter_control.rs")
            && GUTTER_FORBIDDEN_METHODS
                .iter()
                .any(|method| node.method == *method)
        {
            self.push_violation(node.method.span(), GUTTER_SOURCE_ENUMERATION_MESSAGE);
        }
        if self.in_direct_show && node.method == "show_kuc_command_chrome" {
            self.push_violation(node.method.span(), LEGACY_COMMAND_CHROME_SHOW_MESSAGE);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
    fn visit_expr_field(&mut self, node: &'ast syn::ExprField) {
        if self.is_file("gutter_control.rs")
            && matches!(&node.member, syn::Member::Named(member) if member == "latest_kuc_frame")
        {
            self.consumes_latest_kuc_gutter_frame = true;
        }
        syn::visit::visit_expr_field(self, node);
    }
    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        if self.is_file("gutter_control.rs") && node.ident.to_string().ends_with("GutterModel") {
            self.push_violation(node.ident.span(), GUTTER_LOCAL_MODEL_MESSAGE);
        }
        syn::visit::visit_item_struct(self, node);
    }
    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let previous_direct_show = self.in_direct_show;
        let previous_binding_show_count = self.direct_binding_show_count;
        self.in_direct_show = self.is_file("widget.rs")
            && node.sig.ident == "show"
            && matches!(node.vis, syn::Visibility::Public(_));
        self.direct_binding_show_count = 0;
        syn::visit::visit_impl_item_fn(self, node);
        if self.in_direct_show && self.direct_binding_show_count != 1 {
            self.push_violation(node.sig.ident.span(), DIRECT_KUC_SHOW_COUNT_MESSAGE);
        }
        self.in_direct_show = previous_direct_show;
        self.direct_binding_show_count = previous_binding_show_count;
    }
}
#[cfg(test)]
#[path = "kuc_text_runtime_tests.rs"]
mod tests;
