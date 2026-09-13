use super::architecture::{EGUI_CRATE, FLOEM_CRATE, LIB_CRATE};
use super::prohibited_color_literal_patterns::ColorLiteralPatterns;
use crate::diagnostics::{KleLintError, Violation};
use crate::syntax::AttributeOps;
use crate::workspace::{SourceFile, WorkspaceModel};
use std::path::{Path, PathBuf};
use syn::spanned::Spanned;
use syn::visit::Visit;

#[path = "color_literal_source.rs"]
mod color_literal_source;

use color_literal_source::ColorLiteralSource;

pub struct ProhibitedColorLiteralRule;

impl ProhibitedColorLiteralRule {
    pub fn check(workspace: &WorkspaceModel) -> Result<Vec<Violation>, KleLintError> {
        let mut violations = Vec::new();
        for file in workspace.rust_files() {
            if !Self::is_target_file(workspace.root(), file) {
                continue;
            }
            let mut visitor = ColorLiteralVisitor::new(file.path().to_path_buf(), file.source());
            visitor.visit_file(file.syntax());
            violations.extend(visitor.finish()?);
        }
        Ok(violations)
    }

    fn is_target_file(root: &Path, file: &SourceFile) -> bool {
        let path = file.path();
        [LIB_CRATE, EGUI_CRATE, FLOEM_CRATE]
            .iter()
            .map(|crate_path| root.join(crate_path).join("src"))
            .any(|src| path.starts_with(src))
    }
}

struct ColorLiteralVisitor<'source> {
    source: ColorLiteralSource<'source>,
    test_depth: usize,
    violations: Vec<Violation>,
    error: Option<KleLintError>,
}

impl<'source> ColorLiteralVisitor<'source> {
    fn new(file: PathBuf, source: &'source str) -> Self {
        Self {
            source: ColorLiteralSource::new(file, source),
            test_depth: 0,
            violations: Vec::new(),
            error: None,
        }
    }

    fn finish(self) -> Result<Vec<Violation>, KleLintError> {
        self.error.map_or(Ok(self.violations), Err)
    }

    fn push_violation(&mut self, span: proc_macro2::Span, message: &str, hint: &str) {
        if self.test_depth > 0 || self.error.is_some() {
            return;
        }
        let literal = match self.source.capture(span) {
            Ok(literal) => literal,
            Err(error) => {
                self.error = Some(error);
                return;
            }
        };
        self.violations.push(
            Violation::new(
                self.source.file().to_path_buf(),
                span.start().line,
                span.start().column + 1,
                "prohibited-color-literal",
                message,
            )
            .with_literal_hint(literal, hint),
        );
    }

    fn check_path(&mut self, path: &syn::Path) {
        let names = ColorLiteralPatterns::path_segments(path);
        let Some(last) = names.last() else {
            return;
        };
        if ColorLiteralPatterns::is_color_constant_path(&names, last) {
            self.push_violation(
                path.span(),
                "KUC owns host presentation color resolution; KLE must not own color constants.",
                "Remove the KLE color constant reference and use the KUC-owned presentation color.",
            );
        }
    }

    fn check_call(&mut self, call: &syn::ExprCall) {
        let syn::Expr::Path(path) = call.func.as_ref() else {
            return;
        };
        let names = ColorLiteralPatterns::path_segments(&path.path);
        if !ColorLiteralPatterns::is_color_constructor_path(&names)
            || !call.args.iter().any(ColorLiteralPatterns::is_literal_arg)
        {
            return;
        }
        self.push_violation(
            call.span(),
            "KUC owns host presentation color resolution; KLE must not construct literal colors.",
            "Remove the KLE literal color constructor and use the KUC-owned presentation color.",
        );
    }

    fn check_lit(&mut self, lit: &syn::Lit) {
        let syn::Lit::Str(value) = lit else {
            return;
        };
        let value = value.value();
        if ColorLiteralPatterns::is_color_string(&value) {
            self.push_violation(
                lit.span(),
                "KUC owns host presentation color resolution; KLE must not own color literals.",
                "Remove the KLE color literal and use the KUC-owned presentation color.",
            );
        }
    }
}

impl<'ast> Visit<'ast> for ColorLiteralVisitor<'_> {
    fn visit_item_mod(&mut self, node: &'ast syn::ItemMod) {
        if AttributeOps::has_cfg_test_attr(&node.attrs) {
            self.test_depth += 1;
            syn::visit::visit_item_mod(self, node);
            self.test_depth -= 1;
            return;
        }
        syn::visit::visit_item_mod(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        self.check_path(&node.path);
        syn::visit::visit_expr_path(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        self.check_call(node);
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_lit(&mut self, node: &'ast syn::Lit) {
        self.check_lit(node);
        syn::visit::visit_lit(self, node);
    }
}

#[cfg(test)]
#[path = "prohibited_color_literal_tests.rs"]
mod prohibited_color_literal_tests;
#[cfg(test)]
#[path = "prohibited_color_scope_tests.rs"]
mod prohibited_color_scope_tests;
