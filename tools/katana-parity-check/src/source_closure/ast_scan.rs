mod actions;
use std::collections::BTreeSet;

mod definitions;
mod input_candidates;
mod recording;
mod resolution;
mod state;
mod visit;

pub(super) use state::SourceClosureVisitor;

pub(super) fn type_shape_for_index(ty: &syn::Type) -> String {
    SourceClosureVisitor::type_shape(ty)
}

pub(super) fn inherent_member_names(node: &syn::ItemImpl) -> BTreeSet<String> {
    if node.trait_.is_some() {
        return BTreeSet::new();
    }
    node.items
        .iter()
        .filter_map(|item| match item {
            syn::ImplItem::Fn(function) => Some(function.sig.ident.to_string()),
            syn::ImplItem::Const(constant) => Some(constant.ident.to_string()),
            _ => None,
        })
        .collect()
}
