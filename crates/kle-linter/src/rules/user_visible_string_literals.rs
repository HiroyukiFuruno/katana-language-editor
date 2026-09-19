use crate::diagnostics::{KleLintError, Violation};
use crate::span::SpanOps;
use crate::syntax::AttributeOps;
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

use super::architecture::{EGUI_CRATE, FLOEM_CRATE, LIB_CRATE};

pub struct UserVisibleStringLiteralRule;

impl UserVisibleStringLiteralRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !Self::is_target_file(workspace.root(), file) {
                continue;
            }
            let mut visitor = UserVisibleStringVisitor::new(file.path().to_path_buf());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.into_violations());
        }
        Ok(violations)
    }

    fn is_target_file(root: &Path, file: &SourceFile) -> bool {
        let path = file.path();
        if Self::is_test_or_preset_path(path) {
            return false;
        }
        [LIB_CRATE, EGUI_CRATE, FLOEM_CRATE]
            .iter()
            .map(|crate_path| root.join(crate_path).join("src"))
            .any(|src| path.starts_with(src))
    }

    fn is_test_or_preset_path(path: &Path) -> bool {
        path.components().any(|it| {
            let name = it.as_os_str().to_string_lossy();
            name == "tests" || name == "kdv-presets"
        })
    }
}

struct UserVisibleStringVisitor {
    file: PathBuf,
    test_depth: usize,
    violations: Vec<Violation>,
}

impl UserVisibleStringVisitor {
    fn new(file: PathBuf) -> Self {
        Self {
            file,
            test_depth: 0,
            violations: Vec::new(),
        }
    }

    fn into_violations(self) -> Vec<Violation> {
        self.violations
    }

    fn check_method_call(&mut self, call: &syn::ExprMethodCall) {
        let name = call.method.to_string();
        if !Self::is_visible_method(&name) || !call.args.iter().any(Self::is_string_literal) {
            return;
        }
        self.push_violation(call.method.span());
    }

    fn check_call(&mut self, call: &syn::ExprCall) {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        if !Self::is_visible_constructor(&path.path)
            || !call.args.iter().any(Self::is_string_literal)
        {
            return;
        }
        self.push_violation(path.path.span());
    }

    fn push_violation(&mut self, span: proc_macro2::Span) {
        if self.test_depth > 0 {
            return;
        }
        let location = SpanOps::start(span);
        self.violations.push(Violation::new(
            self.file.clone(),
            location.line,
            location.column,
            "prohibited-user-visible-string-literal",
            "user-visible strings must come from injected Strings.",
        ));
    }

    fn is_visible_method(name: &str) -> bool {
        matches!(
            name,
            "label"
                | "button"
                | "checkbox"
                | "radio"
                | "heading"
                | "link"
                | "hyperlink"
                | "menu_button"
                | "selectable_label"
                | "collapsing"
                | "hint_text"
        )
    }

    fn is_visible_constructor(path: &syn::Path) -> bool {
        let names = path
            .segments
            .iter()
            .map(|it| it.ident.to_string())
            .collect::<Vec<_>>();
        let Some(last) = names.last() else {
            return false;
        };
        last == "new"
            && names.iter().any(|it| {
                matches!(
                    it.as_str(),
                    "Button" | "Checkbox" | "Label" | "RichText" | "Window"
                )
            })
    }

    fn is_string_literal(arg: &syn::Expr) -> bool {
        matches!(arg, syn::Expr::Lit(lit) if matches!(lit.lit, syn::Lit::Str(_)))
    }
}

impl<'ast> Visit<'ast> for UserVisibleStringVisitor {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if AttributeOps::has_cfg_test_attr(&node.attrs) {
            self.test_depth += 1;
            syn::visit::visit_item_mod(self, node);
            self.test_depth -= 1;
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        self.check_method_call(node);
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call(node);
        syn::visit::visit_expr_call(self, node);
    }
}

#[cfg(test)]
#[path = "user_visible_string_literals_tests.rs"]
mod user_visible_string_literals_tests;
