use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
#[path = "clipboard_host_acquisition_patterns.rs"]
mod patterns;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

const RULE: &str = "clipboard-host-acquisition";
const BOUNDARY: &str = "KatanA host must own OS clipboard, image, and file-URL acquisition; KLE may pass neutral typed intents and opaque URLs.";
const CORE_FILES: &[&str] = &["clipboard.rs", "controls.rs"];
const EGUI_FILES: &[&str] = &[
    "clipboard_paste_control.rs",
    "kuc_text_surface_mapping.rs",
    "kuc_text_surface_binding.rs",
    "widget_output.rs",
];

pub(super) struct ClipboardHostAcquisitionRule;

impl ClipboardHostAcquisitionRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace
            .rust_files()
            .iter()
            .filter(|file| is_target_file(file.path()))
        {
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }
}

fn is_target_file(path: &Path) -> bool {
    in_directory(path, "crates/katana-language-editor/src", CORE_FILES)
        || in_directory(path, "crates/katana-language-editor-egui/src", EGUI_FILES)
}

fn in_directory(path: &Path, directory: &str, files: &[&str]) -> bool {
    path.parent()
        .is_some_and(|parent| parent.ends_with(directory))
        && path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| files.contains(&name))
}

struct Visitor {
    file: PathBuf,
    violations: Vec<Violation>,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn push(&mut self, span: proc_macro2::Span, category: &str) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            RULE,
            format!("{BOUNDARY} Forbidden {category} implementation."),
        ));
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        if patterns::use_tree_has_root(&node.tree, "arboard") {
            self.push(node.tree.span(), "arboard clipboard access");
        } else if patterns::use_tree_has_host_crate(&node.tree) {
            self.push(node.tree.span(), "direct KatanA crate reference");
        }
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if patterns::path_has_root(node, "arboard") {
            self.push(node.span(), "arboard clipboard access");
        } else if patterns::path_has_host_crate(node) {
            self.push(node.span(), "direct KatanA crate reference");
        } else if patterns::is_filesystem_api(node) {
            self.push(node.span(), "filesystem image-byte acquisition or saving");
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_ident(&mut self, node: &'ast syn::Ident) {
        let name = node.to_string().to_ascii_lowercase();
        if patterns::is_file_url_helper(&name) {
            self.push(node.span(), "file URL to path conversion");
        } else if patterns::is_image_extension_helper(&name) {
            self.push(node.span(), "supported image extension decision");
        }
        syn::visit::visit_ident(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if patterns::is_macos_clipboard_command(node) {
            self.push(node.func.span(), "macOS clipboard process invocation");
        } else if patterns::is_file_url_parse_call(node) {
            self.push(node.func.span(), "file:// URL parsing or conversion");
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if patterns::is_macos_clipboard_method_call(node) {
            self.push(node.method.span(), "macOS clipboard process invocation");
        } else if node.method == "parse" && patterns::is_file_url_literal(&node.receiver) {
            self.push(node.method.span(), "file:// URL parsing or conversion");
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
#[path = "clipboard_host_acquisition_tests.rs"]
mod tests;
