use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

#[path = "action_host_boundary_patterns.rs"]
mod patterns;

const RULE: &str = "action-host-boundary";
const MESSAGE: &str = "KLE and Storybook must forward KUC events opaquely; KatanA action contracts, host mappings, payloads, and action assertions belong to the fixed host.";

pub(super) struct ActionHostBoundaryRule;

impl ActionHostBoundaryRule {
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
    semantic_depth: usize,
}

impl Visitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            violations: Vec::new(),
            semantic_depth: 0,
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
        let semantic = patterns::is_action_type(name)
            || patterns::is_semantic_construct_name(name)
            || patterns::is_action_mapping_name(name);
        if patterns::is_action_type(name)
            || patterns::is_semantic_construct_name(name)
            || patterns::is_action_mapping_name(name)
        {
            self.reject(span, name);
        }
        if semantic {
            self.semantic_depth += 1;
        }
        visit(self);
        if semantic {
            self.semantic_depth -= 1;
        }
    }

    fn check_identifier(&mut self, name: &str, span: proc_macro2::Span) {
        if self.semantic_depth > 0 && patterns::has_host_term(name) {
            self.reject(span, name);
        }
        if patterns::is_action_operation_name(name) {
            self.reject(span, name);
        }
    }
}

impl<'ast> Visit<'ast> for Visitor {
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

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let name = node.ident.to_string();
        self.visit_named(&name, node.ident.span(), |visitor| {
            syn::visit::visit_item_trait(visitor, node)
        });
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        let name = node.ident.to_string();
        self.visit_named(&name, node.ident.span(), |visitor| {
            syn::visit::visit_item_type(visitor, node)
        });
    }

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

    fn visit_field(&mut self, node: &'ast syn::Field) {
        if let Some(name) = &node.ident {
            self.check_identifier(&name.to_string(), name.span());
        }
        syn::visit::visit_field(self, node);
    }

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.check_identifier(&node.ident.to_string(), node.ident.span());
        syn::visit::visit_pat_ident(self, node);
    }

    fn visit_path(&mut self, node: &'ast syn::Path) {
        let mut segments = node.segments.iter().rev();
        if let Some(member) = segments.next()
            && let Some(owner) = segments.next()
        {
            let owner_name = owner.ident.to_string();
            let member_name = member.ident.to_string();
            if patterns::is_editor_action_path(&owner_name)
                || patterns::is_action_event(&owner_name, &member_name)
            {
                self.reject(node.span(), format!("{owner_name}::{member_name}"));
            }
        }
        syn::visit::visit_path(self, node);
    }

    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        let name = node
            .path
            .segments
            .last()
            .map(|segment| segment.ident.to_string());
        if matches!(
            name.as_deref(),
            Some("matches" | "assert" | "assert_matches")
        ) {
            let tokens = node.tokens.to_string();
            if tokens.contains("EditorAction") || tokens.contains("ActionRequested") {
                self.reject(
                    node.span(),
                    name.unwrap_or_else(|| "action assertion".to_string()),
                );
            }
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_macro(&mut self, node: &'ast syn::ExprMacro) {
        self.visit_macro(&node.mac);
    }
}

#[cfg(test)]
#[path = "action_host_boundary_tests.rs"]
mod tests;
