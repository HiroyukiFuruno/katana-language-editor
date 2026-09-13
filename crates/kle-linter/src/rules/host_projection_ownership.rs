use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::workspace::WorkspaceModel;
use std::path::{Path, PathBuf};
use syn::ext::IdentExt;
use syn::visit::Visit;

const RULE: &str = "host-projection-ownership";
const MESSAGE: &str = "HostProjectionProvider and HostProjectionProviderError are owned by the neutral host_projection module.";
const PROVIDER: &str = "HostProjectionProvider";
const PROVIDER_ERROR: &str = "HostProjectionProviderError";

pub(super) struct HostProjectionOwnershipRule;

impl HostProjectionOwnershipRule {
    pub(super) fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if is_allowed_owner(workspace.root(), file.path()) {
                continue;
            }
            let mut visitor = Visitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.violations);
        }
        Ok(violations)
    }
}

fn is_allowed_owner(root: &Path, path: &Path) -> bool {
    path == root.join("crates/katana-language-editor/src/host_projection.rs")
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

    fn check_definition(&mut self, ident: &syn::Ident) {
        let name = ident.unraw().to_string();
        if !matches!(name.as_str(), PROVIDER | PROVIDER_ERROR) {
            return;
        }
        let location = SpanOps::start(ident.span());
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            RULE,
            format!("{MESSAGE} forbidden definition `{name}`."),
        ));
    }
}

impl<'ast> Visit<'ast> for Visitor {
    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        self.check_definition(&node.ident);
        syn::visit::visit_item_trait(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast syn::ItemEnum) {
        self.check_definition(&node.ident);
        syn::visit::visit_item_enum(self, node);
    }

    fn visit_item_struct(&mut self, node: &'ast syn::ItemStruct) {
        self.check_definition(&node.ident);
        syn::visit::visit_item_struct(self, node);
    }

    fn visit_item_type(&mut self, node: &'ast syn::ItemType) {
        self.check_definition(&node.ident);
        syn::visit::visit_item_type(self, node);
    }
}

#[cfg(test)]
#[path = "host_projection_ownership_tests.rs"]
mod tests;
