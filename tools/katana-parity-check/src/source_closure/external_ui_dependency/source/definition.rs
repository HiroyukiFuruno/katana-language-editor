use std::collections::{BTreeMap, BTreeSet};

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{ItemEnum, ItemImpl, Type};

use super::super::super::edge_model::ExternalUiSymbolSpan;
use super::super::REQUIRED;

#[derive(Default)]
pub struct DependencyDefinitionVisitor {
    pub(crate) symbols: BTreeSet<String>,
    pub(crate) spans: BTreeMap<String, Span>,
}

impl DependencyDefinitionVisitor {
    fn add(&mut self, symbol: String, span: Span) {
        self.spans.entry(symbol.clone()).or_insert(span);
        self.symbols.insert(symbol);
    }

    pub(crate) fn impl_type(node: &ItemImpl) -> Option<String> {
        let Type::Path(path) = node.self_ty.as_ref() else {
            return None;
        };
        path.path
            .segments
            .last()
            .map(|segment| segment.ident.to_string())
    }
}

impl<'ast> Visit<'ast> for DependencyDefinitionVisitor {
    fn visit_item_impl(&mut self, node: &'ast ItemImpl) {
        let Some(impl_type) = Self::impl_type(node) else {
            syn::visit::visit_item_impl(self, node);
            return;
        };
        if impl_type == "TextEdit" || impl_type == "InputState" {
            for item in &node.items {
                if let syn::ImplItem::Fn(function) = item {
                    self.add(
                        format!("{impl_type}::{}", function.sig.ident),
                        function.span(),
                    );
                }
            }
        }
        syn::visit::visit_item_impl(self, node);
    }

    fn visit_item_enum(&mut self, node: &'ast ItemEnum) {
        let Some(variant) = node
            .variants
            .iter()
            .find(|variant| node.ident == "Event" && variant.ident == "Paste")
        else {
            syn::visit::visit_item_enum(self, node);
            return;
        };
        self.add("Event::Paste".into(), variant.span());
        syn::visit::visit_item_enum(self, node);
    }
}

pub(super) fn add_required_span(
    visitor: &DependencyDefinitionVisitor,
    symbol: &str,
    relative: &str,
    symbol_spans: &mut Vec<ExternalUiSymbolSpan>,
) {
    let Some(span) = visitor.spans.get(symbol) else {
        return;
    };
    if !REQUIRED.contains(&symbol) {
        return;
    }
    symbol_spans.push(ExternalUiSymbolSpan {
        symbol: symbol.into(),
        span: format!(
            "egui:{relative}:{}:{}-{}:{}",
            span.start().line,
            span.start().column,
            span.end().line,
            span.end().column
        ),
    });
}
