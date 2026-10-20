use std::collections::{BTreeMap, BTreeSet};

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, ExprCall, ExprClosure, ExprMethodCall, Local};

use super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;
use super::SemanticIndexes;
use super::method_call;

pub(super) struct DirectEdgeVisitor<'a> {
    path: &'a str,
    from_symbol: &'a str,
    indexes: &'a SemanticIndexes,
    free_functions: &'a BTreeSet<String>,
    binding_types: BTreeMap<String, String>,
    edges: Vec<ExternalUiDirectSemanticEdge>,
}

impl<'a> DirectEdgeVisitor<'a> {
    pub(super) fn new(
        path: &'a str,
        from_symbol: &'a str,
        indexes: &'a SemanticIndexes,
        free_functions: &'a BTreeSet<String>,
        signature: &syn::Signature,
    ) -> Self {
        Self {
            path,
            from_symbol,
            indexes,
            free_functions,
            binding_types: method_call::parameter_types_with_definitions(
                signature,
                &indexes.defined_types,
            ),
            edges: Vec::new(),
        }
    }

    pub(super) fn visit_into(
        mut self,
        block: &syn::Block,
        edges: &mut Vec<ExternalUiDirectSemanticEdge>,
    ) {
        self.visit_block(block);
        edges.extend(self.edges);
    }

    fn add(&mut self, kind: &str, target_symbol: String, span: proc_macro2::Span) {
        if self.indexes.associated.contains(&target_symbol)
            || self.free_functions.contains(&target_symbol)
        {
            self.edges.push(ExternalUiDirectSemanticEdge {
                from_symbol: self.from_symbol.into(),
                kind: kind.into(),
                source_file: format!("egui/{}", self.path),
                span: format!(
                    "egui:{}:{}:{}-{}:{}",
                    self.path,
                    span.start().line,
                    span.start().column,
                    span.end().line,
                    span.end().column
                ),
                target_symbol,
            });
        }
    }
}

impl<'ast> Visit<'ast> for DirectEdgeVisitor<'_> {
    fn visit_block(&mut self, node: &'ast syn::Block) {
        let outer_bindings = self.binding_types.clone();
        syn::visit::visit_block(self, node);
        self.binding_types = outer_bindings;
    }

    fn visit_local(&mut self, node: &'ast Local) {
        syn::visit::visit_local(self, node);
        let Some(name) = method_call::local_binding_name(node) else {
            return;
        };
        self.binding_types.remove(&name);
        let Some(binding) = method_call::local_binding_type(
            node,
            &self.indexes.associated_constructor_types,
            &self.indexes.associated_option_return_types,
            &self.indexes.default_types,
        ) else {
            return;
        };
        self.binding_types.insert(binding.0, binding.1);
    }

    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(path) = node.func.as_ref() {
            let target = path_symbol(&path.path);
            if self.indexes.associated.contains(&target) || self.free_functions.contains(&target) {
                self.add("call", target, node.span());
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if let Some(target) = method_call::target(
            node,
            &self.binding_types,
            &self.indexes.associated,
            &self.indexes.clone_types,
        ) {
            self.add("method_call", target, node.span());
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_closure(&mut self, _node: &'ast ExprClosure) {}
}

fn path_symbol(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
