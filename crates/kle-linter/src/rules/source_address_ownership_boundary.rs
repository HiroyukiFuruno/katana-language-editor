use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

#[path = "source_address_ownership_boundary_patterns.rs"]
mod patterns;

const RULE: &str = "source-address-ownership-boundary";
const MESSAGE: &str = "KLE and Storybook may forward only the opaque KUC host projection lease; source-address state, rendering, actions, and URL/path construction belong outside KLE.";
pub(super) struct SourceAddressOwnershipBoundaryRule;

impl SourceAddressOwnershipBoundaryRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !is_target(file.path()) {
                continue;
            }
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.violations);
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
    source_address_depth: usize,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            source_address_depth: 0,
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

    fn visit_named<T>(&mut self, name: &str, span: proc_macro2::Span, visit: T)
    where
        T: FnOnce(&mut Self),
    {
        if patterns::is_forbidden_symbol(name) {
            self.reject(span, name);
        }
        if patterns::is_url_conversion_helper(name) {
            self.reject(span, name);
        }
        let enters_source_address = patterns::is_source_address_context(name);
        if enters_source_address {
            self.source_address_depth += 1;
        }
        visit(self);
        if enters_source_address {
            self.source_address_depth -= 1;
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let name = node.sig.ident.to_string();
        self.visit_named(&name, node.sig.ident.span(), |visitor| {
            syn::visit::visit_item_fn(visitor, node)
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        self.visit_named(&name, node.sig.ident.span(), |visitor| {
            syn::visit::visit_impl_item_fn(visitor, node)
        });
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
        visit_use_tree(self, &node.tree);
        syn::visit::visit_item_use(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let names: Vec<String> = node
            .segments
            .iter()
            .map(|segment| segment.ident.to_string())
            .collect();
        let has_forbidden_symbol = names.iter().any(|name| patterns::is_forbidden_symbol(name));
        if has_forbidden_symbol {
            if let Some(name) = names
                .iter()
                .find(|name| patterns::is_forbidden_symbol(name))
            {
                self.reject(node.span(), name);
            }
        } else if patterns::is_url_parse_or_conversion_path(&names)
            || (self.source_address_depth > 0 && patterns::is_direct_source_egui_path(&names))
        {
            self.reject(node.span(), names.join("::"));
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if self.source_address_depth > 0
            && matches!(
                node.method.to_string().as_str(),
                "button" | "text_edit_singleline" | "text_edit_multiline" | "add"
            )
        {
            self.reject(node.method.span(), &node.method);
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_lit(&mut self, node: &'ast syn::ExprLit) {
        if let syn::Lit::Str(value) = &node.lit
            && value.value().contains("file://")
        {
            self.reject(value.span(), "file:// literal");
        }
        syn::visit::visit_expr_lit(self, node);
    }
}

fn visit_use_tree(visitor: &mut Visitor, tree: &syn::UseTree) {
    match tree {
        syn::UseTree::Path(path) => visit_use_tree(visitor, &path.tree),
        syn::UseTree::Name(name) => {
            if patterns::is_forbidden_symbol(&name.ident.to_string()) {
                visitor.reject(name.ident.span(), &name.ident);
            }
        }
        syn::UseTree::Rename(rename) => {
            if patterns::is_forbidden_symbol(&rename.ident.to_string()) {
                visitor.reject(rename.ident.span(), &rename.ident);
            }
            if patterns::is_forbidden_symbol(&rename.rename.to_string()) {
                visitor.reject(rename.rename.span(), &rename.rename);
            }
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                visit_use_tree(visitor, item);
            }
        }
        syn::UseTree::Glob(_) => {}
    }
}

#[cfg(test)]
#[path = "source_address_ownership_boundary_tests.rs"]
mod tests;
