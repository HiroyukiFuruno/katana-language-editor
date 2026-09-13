use std::collections::BTreeSet;

use syn::{ImplItem, Item, ItemImpl};

use super::super::super::edge_model::ExternalUiDirectSemanticEdge;

const TEXT_EDIT_SEEDS: [&str; 3] = [
    "TextEdit::show",
    "TextEdit::load_state",
    "TextEdit::store_state",
];

mod closure_callback;
mod context_receiver;
mod index;
mod method_call;
mod reference_receiver;
mod visitor;

use index::{
    associated_constructor_types, associated_option_return_types, associated_symbols,
    defined_type_names, derived_types, file_free_functions,
};
use visitor::DirectEdgeVisitor;

pub(super) fn collect(sources: &[(String, syn::File)]) -> Vec<ExternalUiDirectSemanticEdge> {
    let indexes = SemanticIndexes::from_sources(sources);
    let mut edges = reference_receiver::collect(sources);
    edges.extend(closure_callback::collect(sources));
    edges.extend(context_receiver::collect(sources));
    for (path, file) in sources {
        collect_file_edges(path, file, &indexes, &mut edges);
    }
    edges.sort_by(|left, right| {
        (
            left.source_file.as_str(),
            left.span.as_str(),
            left.from_symbol.as_str(),
            left.target_symbol.as_str(),
        )
            .cmp(&(
                right.source_file.as_str(),
                right.span.as_str(),
                right.from_symbol.as_str(),
                right.target_symbol.as_str(),
            ))
    });
    edges.dedup_by(|left, right| {
        left.from_symbol == right.from_symbol
            && left.kind == right.kind
            && left.source_file == right.source_file
            && left.span == right.span
            && left.target_symbol == right.target_symbol
    });
    edges
}

struct SemanticIndexes {
    associated: BTreeSet<String>,
    associated_constructor_types: std::collections::BTreeMap<String, String>,
    associated_option_return_types: std::collections::BTreeMap<String, String>,
    defined_types: BTreeSet<String>,
    default_types: BTreeSet<String>,
    clone_types: BTreeSet<String>,
}

impl SemanticIndexes {
    fn from_sources(sources: &[(String, syn::File)]) -> Self {
        Self {
            associated: associated_symbols(sources),
            associated_constructor_types: associated_constructor_types(sources),
            associated_option_return_types: associated_option_return_types(sources),
            defined_types: defined_type_names(sources),
            default_types: derived_types(sources, "Default"),
            clone_types: derived_types(sources, "Clone"),
        }
    }
}

fn collect_file_edges(
    path: &str,
    file: &syn::File,
    indexes: &SemanticIndexes,
    edges: &mut Vec<ExternalUiDirectSemanticEdge>,
) {
    let free_functions = file_free_functions(file);
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        let Some(type_name) = impl_type(item_impl) else {
            continue;
        };
        if type_name != "TextEdit" {
            continue;
        }
        for item in &item_impl.items {
            let ImplItem::Fn(function) = item else {
                continue;
            };
            let from_symbol = format!("TextEdit::{}", function.sig.ident);
            if TEXT_EDIT_SEEDS.contains(&from_symbol.as_str()) {
                DirectEdgeVisitor::new(path, &from_symbol, indexes, &free_functions, &function.sig)
                    .visit_into(&function.block, edges);
            }
        }
    }
}

fn impl_type(item: &ItemImpl) -> Option<String> {
    let syn::Type::Path(path) = item.self_ty.as_ref() else {
        return None;
    };
    path.path
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

#[cfg(test)]
#[path = "semantic_tests.rs"]
mod semantic_tests;
