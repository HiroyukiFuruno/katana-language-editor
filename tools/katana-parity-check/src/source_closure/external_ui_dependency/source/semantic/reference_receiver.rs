use std::collections::BTreeSet;

use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{Expr, FnArg, ImplItem, Item, Pat, Stmt, Type};

use super::super::super::super::edge_model::ExternalUiDirectSemanticEdge;

mod definition;
#[cfg(test)]
mod tests;

pub(super) fn collect(sources: &[(String, syn::File)]) -> Vec<ExternalUiDirectSemanticEdge> {
    if !definition::has_unique_ui_ctx(sources) {
        return Vec::new();
    }
    let mut edges = Vec::new();
    for (path, file) in sources {
        for item in &file.items {
            let Item::Impl(item) = item else { continue };
            if item.trait_.is_some()
                || !item.generics.params.is_empty()
                || item.generics.where_clause.is_some()
                || !definition::named_type(&item.self_ty, "TextEdit")
            {
                continue;
            }
            collect_impl(path, item, &mut edges);
        }
    }
    edges
}

fn collect_impl(path: &str, item: &syn::ItemImpl, edges: &mut Vec<ExternalUiDirectSemanticEdge>) {
    for member in &item.items {
        let ImplItem::Fn(function) = member else {
            continue;
        };
        if function.sig.ident != "show"
            || !function.sig.generics.params.is_empty()
            || function.sig.generics.where_clause.is_some()
        {
            continue;
        }
        collect_function(path, function, edges);
    }
}

fn collect_function(
    path: &str,
    function: &syn::ImplItemFn,
    edges: &mut Vec<ExternalUiDirectSemanticEdge>,
) {
    let mut bindings = reference_parameters(&function.sig);
    for statement in &function.block.stmts {
        let mut hazards = Hazards::default();
        hazards.visit_stmt(statement);
        let unsupported_item = match statement {
            Stmt::Item(Item::Const(item)) => bindings.contains(&item.ident.to_string()),
            Stmt::Item(_) => true,
            _ => false,
        };
        if hazards.has_macro || unsupported_item {
            break;
        }
        let expression = match statement {
            Stmt::Local(local) => local.init.as_ref().map(|init| init.expr.as_ref()),
            Stmt::Expr(expression, _) => Some(expression),
            _ => None,
        };
        if let Some(expression) = expression {
            Calls {
                path,
                bindings: &bindings,
                edges,
            }
            .visit_expr(expression);
        }
        if let Stmt::Local(local) = statement {
            let mut names = BoundNames::default();
            names.visit_pat(&local.pat);
            for name in names.0 {
                bindings.remove(&name);
            }
        }
    }
}

fn reference_parameters(signature: &syn::Signature) -> BTreeSet<String> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| {
            let FnArg::Typed(argument) = argument else {
                return None;
            };
            let Pat::Ident(binding) = argument.pat.as_ref() else {
                return None;
            };
            if binding.by_ref.is_some() || binding.subpat.is_some() {
                return None;
            }
            let Type::Reference(reference) = argument.ty.as_ref() else {
                return None;
            };
            definition::plain_named_type(&reference.elem, "Ui").then(|| binding.ident.to_string())
        })
        .collect()
}

#[derive(Default)]
struct BoundNames(BTreeSet<String>);
impl<'ast> Visit<'ast> for BoundNames {
    fn visit_pat_ident(&mut self, node: &'ast syn::PatIdent) {
        self.0.insert(node.ident.to_string());
        syn::visit::visit_pat_ident(self, node);
    }
}

#[derive(Default)]
struct Hazards {
    has_macro: bool,
}
impl<'ast> Visit<'ast> for Hazards {
    fn visit_macro(&mut self, _: &'ast syn::Macro) {
        self.has_macro = true;
    }
}

struct Calls<'a> {
    path: &'a str,
    bindings: &'a BTreeSet<String>,
    edges: &'a mut Vec<ExternalUiDirectSemanticEdge>,
}

impl<'ast> Visit<'ast> for Calls<'_> {
    fn visit_expr(&mut self, node: &'ast Expr) {
        /* WHY: 分岐・遅延実行・別スコープの名前解決はこの経路の証明対象外。 */
        if matches!(
            node,
            Expr::Block(_)
                | Expr::Closure(_)
                | Expr::If(_)
                | Expr::Match(_)
                | Expr::ForLoop(_)
                | Expr::While(_)
                | Expr::Loop(_)
                | Expr::Async(_)
                | Expr::Const(_)
                | Expr::Unsafe(_)
        ) {
            return;
        }
        syn::visit::visit_expr(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if node.method == "ctx"
            && node.args.is_empty()
            && node.turbofish.is_none()
            && let Expr::Path(receiver) = node.receiver.as_ref()
            && receiver.qself.is_none()
            && let Some(name) = receiver.path.get_ident()
            && self.bindings.contains(&name.to_string())
        {
            let span = node.span();
            self.edges.push(ExternalUiDirectSemanticEdge {
                from_symbol: "TextEdit::show".into(),
                target_symbol: "Ui::ctx".into(),
                kind: "method_call".into(),
                source_file: format!("egui/{}", self.path),
                span: format!(
                    "egui:{}:{}:{}-{}:{}",
                    self.path,
                    span.start().line,
                    span.start().column,
                    span.end().line,
                    span.end().column
                ),
            });
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}
