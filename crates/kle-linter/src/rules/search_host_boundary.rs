use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;
#[path = "search_host_boundary_patterns.rs"]
mod patterns;
use patterns::{
    SEARCH, has_word, is_constant_identifier, is_forbidden_operation_name, is_local_semantic_name,
    is_low_level_editing_name, mutation,
};

const RULE: &str = "search-host-boundary";
const MESSAGE: &str = "KLE and Storybook must forward generic KUC search events opaquely; document matching, regex, ranges, navigation, replacement, and mutation belong to the fixed host.";

pub(super) struct SearchHostBoundaryRule;

impl SearchHostBoundaryRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if is_target(file.path()) {
                let mut visitor = Visitor::new(file.path().to_path_buf());
                visitor.visit_file(file.syntax());
                violations.extend(visitor.violations);
            }
        }
        Ok(violations)
    }
}

fn is_target(path: &Path) -> bool {
    let path = path.to_string_lossy();
    path.contains("crates/katana-language-editor/src/")
        || path.contains("crates/katana-language-editor-egui/src/")
        || path.contains("tools/kle-storybook/src/")
}

struct Visitor {
    file: PathBuf,
    violations: Vec<Violation>,
    in_search_code: bool,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            in_search_code: false,
        }
    }

    fn reject(&mut self, span: proc_macro2::Span, found: impl std::fmt::Display) {
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            RULE,
            format!("{MESSAGE} forbidden `{found}`."),
        ));
    }

    fn body<T>(&mut self, span: proc_macro2::Span, name: &str, visit: T)
    where
        T: FnOnce(&mut Self),
    {
        let old_search = self.in_search_code;
        self.in_search_code |= !is_low_level_editing_name(name) && has_word(name, SEARCH);
        if is_forbidden_operation_name(name) {
            self.reject(span, name);
        }
        visit(self);
        self.in_search_code = old_search;
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item(&mut self, node: &'ast syn::Item) {
        match node {
            syn::Item::Fn(item) => {
                let name = item.sig.ident.to_string();
                self.body(item.sig.ident.span(), &name, |visitor| {
                    syn::visit::visit_item_fn(visitor, item)
                });
                return;
            }
            syn::Item::Struct(item) if is_local_semantic_name(&item.ident.to_string()) => {
                self.reject(item.ident.span(), &item.ident)
            }
            syn::Item::Enum(item) if is_local_semantic_name(&item.ident.to_string()) => {
                self.reject(item.ident.span(), &item.ident)
            }
            _ => {}
        }
        syn::visit::visit_item(self, node);
    }

    fn visit_impl_item(&mut self, node: &'ast syn::ImplItem) {
        if let syn::ImplItem::Fn(item) = node {
            let name = item.sig.ident.to_string();
            self.body(item.sig.ident.span(), &name, |visitor| {
                syn::visit::visit_impl_item_fn(visitor, item)
            });
            return;
        }
        syn::visit::visit_impl_item(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(name) = &node.ident
            && is_local_semantic_name(&name.to_string())
        {
            self.reject(name.span(), name);
        }
        syn::visit::visit_field(self, node);
    }
    fn visit_pat(&mut self, node: &'ast syn::Pat) {
        if let syn::Pat::Ident(pattern) = node
            && !is_constant_identifier(&pattern.ident.to_string())
            && is_local_semantic_name(&pattern.ident.to_string())
        {
            self.reject(pattern.ident.span(), &pattern.ident);
        }
        syn::visit::visit_pat(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let mut segments = node.segments.iter().rev();
        let Some(action) = segments.next() else {
            return;
        };
        let Some(owner) = segments.next() else {
            syn::visit::visit_path(self, node);
            return;
        };
        let action_name = action.ident.to_string();
        if owner.ident == "EditorAction"
            && matches!(
                action_name.as_str(),
                "Find" | "Replace" | "FindNext" | "FindPrevious"
            )
        {
            self.reject(node.span(), format!("{}::{action_name}", owner.ident));
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.in_search_code && mutation(&node.method.to_string()) {
            self.reject(node.method.span(), &node.method);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.in_search_code
            && let syn::Expr::Path(path) = &*node.func
            && let Some(segment) = path.path.segments.last()
            && mutation(&segment.ident.to_string())
        {
            self.reject(node.func.span(), &segment.ident);
        }
        syn::visit::visit_expr_call(self, node);
    }
}
#[cfg(test)]
#[path = "search_host_boundary_tests.rs"]
mod tests;
