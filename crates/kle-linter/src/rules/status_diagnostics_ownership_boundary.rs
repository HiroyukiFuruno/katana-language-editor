use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::visit::Visit;

#[path = "status_diagnostics_ownership_boundary_patterns.rs"]
mod patterns;

pub(super) struct StatusDiagnosticsOwnershipBoundaryRule;

impl StatusDiagnosticsOwnershipBoundaryRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !patterns::is_target(file.path()) || patterns::is_test_source(file.path()) {
                continue;
            }
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.violations);
        }
        Ok(violations)
    }
}

struct Visitor {
    file: PathBuf,
    violations: Vec<Violation>,
    test_depth: usize,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            test_depth: 0,
        }
    }

    fn reject(&mut self, span: proc_macro2::Span, found: impl std::fmt::Display) {
        if self.test_depth > 0 {
            return;
        }
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            patterns::RULE,
            format!("{} forbidden `{found}`.", patterns::MESSAGE),
        ));
    }

    fn check_identifier(&mut self, ident: &syn::Ident) {
        if patterns::is_forbidden_symbol(&ident.to_string()) {
            self.reject(ident.span(), ident);
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if patterns::is_cfg_test_module(&node.attrs) {
            self.test_depth += 1;
            syn::visit::visit_item_mod(self, node);
            self.test_depth -= 1;
        } else {
            syn::visit::visit_item_mod(self, node);
        }
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.check_identifier(&node.sig.ident);
        syn::visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.check_identifier(&node.sig.ident);
        syn::visit::visit_impl_item_fn(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_identifier(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_identifier(&node.ident);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.check_identifier(&node.ident);
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.check_identifier(&node.ident);
        syn::visit::visit_item_type(self, node);
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        patterns::visit_use_tree(&node.tree, &mut |ident| self.check_identifier(ident));
        syn::visit::visit_item_use(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(ident) = &node.ident {
            self.check_identifier(ident);
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.check_identifier(&node.ident);
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        for segment in &node.segments {
            self.check_identifier(&segment.ident);
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_identifier(&node.method);
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
#[path = "status_diagnostics_ownership_boundary_tests.rs"]
mod tests;
