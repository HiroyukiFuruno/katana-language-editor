use std::collections::{BTreeMap, BTreeSet};

use syn::spanned::Spanned;
use syn::visit::{self, Visit};
use syn::{Expr, ExprClosure, ImplItem, Item, ItemImpl, Pat, Stmt};

use super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;

mod edge;

pub(super) fn collect(sources: &[(String, syn::File)]) -> Vec<ExternalUiDirectSemanticEdge> {
    sources
        .iter()
        .flat_map(|(path, file)| collect_file(path, file))
        .collect()
}

fn collect_file(path: &str, file: &syn::File) -> Vec<ExternalUiDirectSemanticEdge> {
    let free_functions = unique_free_functions(file);
    text_edit_show_functions(file)
        .flat_map(|function| callback_edges(path, function, &free_functions))
        .collect()
}

fn unique_free_functions(file: &syn::File) -> BTreeSet<String> {
    let counts = file.items.iter().filter_map(|item| match item {
        Item::Fn(function) => Some(function.sig.ident.to_string()),
        _ => None,
    });
    counts
        .fold(BTreeMap::new(), |mut counts, name| {
            *counts.entry(name).or_insert(0_usize) += 1;
            counts
        })
        .into_iter()
        .filter_map(|(name, count)| (count == 1).then_some(name))
        .collect()
}

fn text_edit_show_functions(file: &syn::File) -> impl Iterator<Item = &syn::ImplItemFn> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) if is_text_edit_impl(item) => Some(item),
            _ => None,
        })
        .flat_map(|item| item.items.iter())
        .filter_map(|item| match item {
            ImplItem::Fn(function)
                if function.sig.ident == "show"
                    && function.sig.generics.params.is_empty()
                    && function.sig.generics.where_clause.is_none() =>
            {
                Some(function)
            }
            _ => None,
        })
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

fn callback_edges(
    path: &str,
    function: &syn::ImplItemFn,
    free_functions: &BTreeSet<String>,
) -> Vec<ExternalUiDirectSemanticEdge> {
    let bindings = closure_bindings(&function.block, free_functions);
    let names = binding_counts(&function.block);
    let invoked = invoked_bindings(&function.block, bindings.keys().cloned().collect());
    bindings
        .into_iter()
        .filter(|(name, _)| names.get(name) == Some(&1) && invoked.contains(name))
        .flat_map(|(_, calls)| calls)
        .map(|(target, span)| edge::create(path, target, span))
        .collect()
}

fn closure_bindings(
    block: &syn::Block,
    free_functions: &BTreeSet<String>,
) -> BTreeMap<String, Vec<(String, proc_macro2::Span)>> {
    block
        .stmts
        .iter()
        .filter_map(|statement| closure_binding(statement))
        .map(|(name, closure)| (name, closure_calls(closure, free_functions)))
        .filter(|(_, calls)| !calls.is_empty())
        .collect()
}

fn closure_binding(statement: &Stmt) -> Option<(String, &ExprClosure)> {
    let Stmt::Local(local) = statement else {
        return None;
    };
    let Pat::Ident(binding) = &local.pat else {
        return None;
    };
    let Expr::Closure(closure) = local.init.as_ref()?.expr.as_ref() else {
        return None;
    };
    (binding.by_ref.is_none() && binding.subpat.is_none())
        .then(|| (binding.ident.to_string(), closure))
}

fn closure_calls(
    closure: &ExprClosure,
    free_functions: &BTreeSet<String>,
) -> Vec<(String, proc_macro2::Span)> {
    let mut visitor = FreeCallVisitor {
        free_functions,
        calls: Vec::new(),
    };
    visitor.visit_expr(&closure.body);
    visitor.calls
}

fn binding_counts(block: &syn::Block) -> BTreeMap<String, usize> {
    let mut visitor = BindingVisitor::default();
    visitor.visit_block(block);
    visitor.counts
}

fn invoked_bindings(block: &syn::Block, bindings: BTreeSet<String>) -> BTreeSet<String> {
    let mut visitor = InvocationVisitor {
        bindings,
        invoked: BTreeSet::new(),
    };
    visitor.visit_block(block);
    visitor.invoked
}

#[derive(Default)]
struct BindingVisitor {
    counts: BTreeMap<String, usize>,
}
impl<'ast> Visit<'ast> for BindingVisitor {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        *self.counts.entry(node.ident.to_string()).or_default() += 1;
        visit::visit_pat_ident(self, node);
    }
    fn visit_expr_closure(&mut self, _: &'ast ExprClosure) {}
}

struct FreeCallVisitor<'a> {
    free_functions: &'a BTreeSet<String>,
    calls: Vec<(String, proc_macro2::Span)>,
}
impl<'ast> Visit<'ast> for FreeCallVisitor<'_> {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = node.func.as_ref()
            && path.qself.is_none()
            && let Some(ident) = path.path.get_ident()
            && self.free_functions.contains(&ident.to_string())
        {
            self.calls.push((ident.to_string(), node.span()));
        }
        visit::visit_expr_call(self, node);
    }
    fn visit_expr_closure(&mut self, _: &'ast ExprClosure) {}
    fn visit_macro(&mut self, _: &'ast syn::Macro) {}
}

struct InvocationVisitor {
    bindings: BTreeSet<String>,
    invoked: BTreeSet<String>,
}
impl<'ast> Visit<'ast> for InvocationVisitor {
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let Expr::Path(path) = node.func.as_ref()
            && path.qself.is_none()
            && let Some(ident) = path.path.get_ident()
            && self.bindings.contains(&ident.to_string())
        {
            self.invoked.insert(ident.to_string());
        }
        visit::visit_expr_call(self, node);
    }
    fn visit_expr_closure(&mut self, _: &'ast ExprClosure) {}
    fn visit_macro(&mut self, _: &'ast syn::Macro) {}
}

#[cfg(test)]
mod tests {
    use super::collect;

    #[test]
    fn records_only_an_invoked_unique_local_callback() -> Result<(), syn::Error> {
        let source = "struct TextEdit; fn events() {} impl TextEdit { fn show() { let callback = || events(); callback(); } }";
        let file = syn::parse_file(source)?;
        let edges = collect(&[("src/widgets/text_edit/builder.rs".into(), file)]);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0].target_symbol, "events");
        Ok(())
    }

    #[test]
    fn rejects_shadowed_and_macro_invocation_routes() -> Result<(), syn::Error> {
        for body in [
            "let callback = || events(); callback(); let callback = || (); callback();",
            "let callback = || events(); invoke!(callback());",
        ] {
            let source = format!(
                "struct TextEdit; fn events() {{}} impl TextEdit {{ fn show() {{ {body} }} }}"
            );
            let file = syn::parse_file(&source)?;
            assert!(collect(&[("src/widgets/text_edit/builder.rs".into(), file)]).is_empty());
        }
        Ok(())
    }
}
