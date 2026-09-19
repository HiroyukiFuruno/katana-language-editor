use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ImplItem, Item, ItemImpl};

use super::super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;
use super::definition;

pub(super) fn collect_file(
    path: &str,
    file: &syn::File,
    ctx: &str,
    targets: &BTreeSet<String>,
) -> Vec<ExternalUiDirectSemanticEdge> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) if is_text_edit_impl(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| item.items.iter())
        .filter_map(|member| match member {
            ImplItem::Fn(function) if function.sig.ident == "show" => Some(function),
            _ => None,
        })
        .filter(|function| function.sig.generics.params.is_empty())
        .filter(|function| function.sig.generics.where_clause.is_none())
        .filter_map(|function| {
            definition::ui_binding(&function.sig).map(|binding| (function, binding))
        })
        .flat_map(|(function, binding)| {
            Calls::new(&binding, ctx, targets).collect(path, &function.block)
        })
        .collect()
}

fn is_text_edit_impl(item: &ItemImpl) -> bool {
    item.trait_.is_none()
        && item.generics.params.is_empty()
        && item.generics.where_clause.is_none()
        && matches!(&*item.self_ty, syn::Type::Path(path)
            if path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0].ident == "TextEdit")
}

struct Calls<'a> {
    binding: &'a str,
    ctx: &'a str,
    targets: &'a BTreeSet<String>,
    shadowed: bool,
    edges: Vec<(String, proc_macro2::Span)>,
}

impl<'a> Calls<'a> {
    fn new(binding: &'a str, ctx: &'a str, targets: &'a BTreeSet<String>) -> Self {
        Self {
            binding,
            ctx,
            targets,
            shadowed: false,
            edges: Vec::new(),
        }
    }

    fn collect(mut self, path: &str, block: &syn::Block) -> Vec<ExternalUiDirectSemanticEdge> {
        self.visit_block(block);
        self.edges
            .into_iter()
            .map(|(target, span)| edge(path, target, span))
            .collect()
    }
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if !self.shadowed
            && self.targets.contains(&node.method.to_string())
            && node.turbofish.is_none()
            && is_ui_ctx_call(node.receiver.as_ref(), self.binding, self.ctx)
        {
            self.edges.push((node.method.to_string(), node.span()));
        }
        visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_closure(&mut self, _: &'ast syn::ExprClosure) {}

    fn visit_macro(&mut self, _: &'ast syn::Macro) {}

    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        if node.ident == self.binding {
            self.shadowed = true;
        }
        visit::visit_pat_ident(self, node);
    }
}

pub(super) fn is_ui_ctx_call(expression: &Expr, binding: &str, ctx: &str) -> bool {
    matches!(expression, Expr::MethodCall(call)
        if call.method == ctx
            && call.args.is_empty()
            && call.turbofish.is_none()
            && matches!(call.receiver.as_ref(), Expr::Path(path)
                if path.qself.is_none()
                    && path.path.get_ident().is_some_and(|ident| ident == binding)))
}

fn edge(path: &str, target: String, span: proc_macro2::Span) -> ExternalUiDirectSemanticEdge {
    ExternalUiDirectSemanticEdge {
        from_symbol: "TextEdit::show".into(),
        target_symbol: format!("Context::{target}"),
        kind: "method_call".into(),
        source_file: format!("egui/{path}"),
        span: format!(
            "egui:{path}:{}:{}-{}:{}",
            span.start().line,
            span.start().column,
            span.end().line,
            span.end().column
        ),
    }
}
