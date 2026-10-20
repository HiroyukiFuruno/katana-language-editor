use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

#[path = "authoring_host_boundary_patterns.rs"]
mod patterns;

const RULE: &str = "authoring-host-boundary";
const MESSAGE: &str = "KLE authoring UI must remain generic KUC presentation plus opaque forwarding; Markdown operations, host actions, transforms, and popup selection state belong to fixed KatanA.";

pub(super) struct AuthoringHostBoundaryRule;

impl AuthoringHostBoundaryRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !is_target(file.path()) {
                continue;
            }
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
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
    authoring_context: usize,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            authoring_context: 0,
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
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

    fn check_name(&mut self, span: proc_macro2::Span, name: &str) {
        let opaque_forwarding = patterns::is_opaque_forwarding_name(name);
        if patterns::is_forbidden_type(name)
            || (!opaque_forwarding && patterns::is_authoring_mapping_name(name))
            || patterns::is_authoring_state_name(name)
            || (self.authoring_context > 0 && patterns::is_state_name(name))
        {
            self.reject(span, name);
        }
    }

    fn visit_named<T>(&mut self, name: &str, span: proc_macro2::Span, visit: T)
    where
        T: FnOnce(&mut Self),
    {
        self.check_name(span, name);
        let enters = patterns::is_authoring_context(name);
        if enters {
            self.authoring_context += 1;
        }
        visit(self);
        if enters {
            self.authoring_context -= 1;
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

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        let name = node.ident.to_string();
        self.visit_named(&name, node.ident.span(), |visitor| {
            syn::visit::visit_item_struct(visitor, node)
        });
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        let name = node.ident.to_string();
        self.visit_named(&name, node.ident.span(), |visitor| {
            syn::visit::visit_item_enum(visitor, node)
        });
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let name = node.sig.ident.to_string();
        self.visit_named(&name, node.sig.ident.span(), |visitor| {
            syn::visit::visit_impl_item_fn(visitor, node)
        });
    }

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(name) = &node.ident {
            self.check_name(name.span(), &name.to_string());
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.check_name(node.ident.span(), &node.ident.to_string());
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        if let Some(segment) = node.segments.last()
            && patterns::is_forbidden_type(&segment.ident.to_string())
        {
            self.reject(node.span(), segment.ident.to_string());
        }
        let mut segments = node.segments.iter().rev();
        if let (Some(action), Some(owner)) = (segments.next(), segments.next())
            && owner.ident == "AppAction"
            && action.ident == "AuthorMarkdown"
        {
            self.reject(node.span(), "AppAction::AuthorMarkdown");
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if patterns::is_authoring_transform_method(&node.method.to_string()) {
            self.reject(node.method.span(), &node.method);
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

#[cfg(test)]
#[path = "authoring_host_boundary_tests.rs"]
mod tests;
