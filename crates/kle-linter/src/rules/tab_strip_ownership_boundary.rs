use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;
#[path = "tab_strip_ownership_boundary_patterns.rs"]
mod patterns;
pub(super) struct TabStripOwnershipBoundaryRule;
impl TabStripOwnershipBoundaryRule {
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
    tab_context_depth: usize,
    test_depth: usize,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            tab_context_depth: 0,
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

    fn check_name(&mut self, name: &str, span: proc_macro2::Span) -> bool {
        let normalized = patterns::normalize(name);
        if patterns::is_forbidden_symbol(&normalized) {
            self.reject(span, name);
        }
        patterns::is_tab_context_name(&normalized)
    }

    fn visit_named<T>(&mut self, name: &str, span: proc_macro2::Span, visit: T)
    where
        T: FnOnce(&mut Self),
    {
        let enters_tab_context = self.check_name(name, span);
        if enters_tab_context {
            self.tab_context_depth += 1;
        }
        visit(self);
        if enters_tab_context {
            self.tab_context_depth -= 1;
        }
    }

    fn check_identifier(&mut self, name: &str, span: proc_macro2::Span) {
        if patterns::is_forbidden_symbol(&patterns::normalize(name)) {
            self.reject(span, name);
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
        self.visit_named(
            &node.sig.ident.to_string(),
            node.sig.ident.span(),
            |visitor| syn::visit::visit_item_fn(visitor, node),
        );
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.visit_named(
            &node.sig.ident.to_string(),
            node.sig.ident.span(),
            |visitor| syn::visit::visit_impl_item_fn(visitor, node),
        );
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.visit_named(&node.ident.to_string(), node.ident.span(), |visitor| {
            syn::visit::visit_item_struct(visitor, node)
        });
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.visit_named(&node.ident.to_string(), node.ident.span(), |visitor| {
            syn::visit::visit_item_enum(visitor, node)
        });
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.visit_named(&node.ident.to_string(), node.ident.span(), |visitor| {
            syn::visit::visit_item_trait(visitor, node)
        });
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.visit_named(&node.ident.to_string(), node.ident.span(), |visitor| {
            syn::visit::visit_item_type(visitor, node)
        });
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        patterns::visit_use_tree(&node.tree, &mut |ident| {
            self.check_identifier(&ident.to_string(), ident.span());
        });
        syn::visit::visit_item_use(self, node);
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(ident) = &node.ident {
            self.check_identifier(&ident.to_string(), ident.span());
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.check_identifier(&node.ident.to_string(), node.ident.span());
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let names: Vec<String> = node
            .segments
            .iter()
            .map(|segment| patterns::normalize(&segment.ident.to_string()))
            .collect();
        if names.iter().any(|name| patterns::is_forbidden_symbol(name)) {
            if let Some(name) = names
                .iter()
                .find(|name| patterns::is_forbidden_symbol(name))
            {
                self.reject(node.span(), name);
            }
        } else if self.tab_context_depth > 0
            && names.first().is_some_and(|name| name == "egui")
            && names
                .iter()
                .any(|name| patterns::is_direct_egui_tab_child(name))
        {
            self.reject(node.span(), names.join("::"));
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        let method = patterns::normalize(&node.method.to_string());
        if self.tab_context_depth > 0 && patterns::is_direct_egui_child_method(&method) {
            self.reject(node.method.span(), &node.method);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        if self.tab_context_depth > 0
            && let syn::Lit::Str(value) = &node.lit
            && patterns::is_raw_color_hex(&value.value())
        {
            self.reject(value.span(), "raw color hex literal");
        }
        syn::visit::visit_expr_lit(self, node);
    }
}

#[cfg(test)]
#[path = "tab_strip_ownership_boundary_tests.rs"]
mod tests;
