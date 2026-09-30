use std::collections::{BTreeMap, BTreeSet};

use syn::{ImplItem, Item, ItemFn, ItemImpl};

mod type_facts;

pub(super) use type_facts::{associated_option_return_types, derived_types};

pub(super) fn defined_type_names(sources: &[(String, syn::File)]) -> BTreeSet<String> {
    let mut counts = BTreeMap::new();
    for (_, file) in sources {
        for item in &file.items {
            let name = match item {
                Item::Enum(item) => item.ident.to_string(),
                Item::Struct(item) => item.ident.to_string(),
                Item::Union(item) => item.ident.to_string(),
                _ => continue,
            };
            *counts.entry(name).or_insert(0usize) += 1;
        }
    }
    counts
        .into_iter()
        .filter_map(|(name, count)| (count == 1).then_some(name))
        .collect()
}

pub(super) fn associated_symbols(sources: &[(String, syn::File)]) -> BTreeSet<String> {
    let mut counts = BTreeMap::new();
    for (_, file) in sources {
        add_file_associated_symbols(file, &mut counts);
    }
    counts
        .into_iter()
        .filter_map(|(symbol, count)| (count == 1).then_some(symbol))
        .collect()
}

pub(super) fn associated_constructor_types(
    sources: &[(String, syn::File)],
) -> BTreeMap<String, String> {
    let mut candidates = BTreeMap::new();
    for (_, file) in sources {
        add_file_constructor_types(file, &mut candidates);
    }
    candidates
        .into_iter()
        .filter_map(|(symbol, mut candidates)| {
            (candidates.len() == 1).then(|| (symbol, candidates.remove(0)))
        })
        .collect()
}

fn add_file_associated_symbols(file: &syn::File, counts: &mut BTreeMap<String, usize>) {
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        add_impl_associated_symbols(item_impl, counts);
    }
}

fn add_file_constructor_types(file: &syn::File, candidates: &mut BTreeMap<String, Vec<String>>) {
    for item in &file.items {
        let Item::Impl(item_impl) = item else {
            continue;
        };
        add_impl_constructor_types(item_impl, candidates);
    }
}

fn add_impl_constructor_types(
    item_impl: &ItemImpl,
    candidates: &mut BTreeMap<String, Vec<String>>,
) {
    if item_impl.trait_.is_some() {
        return;
    }
    let Some(type_name) = impl_type(item_impl) else {
        return;
    };
    for item in &item_impl.items {
        let ImplItem::Fn(function) = item else {
            continue;
        };
        let Some(return_type) = explicit_return_type(&function.sig, &type_name) else {
            continue;
        };
        candidates
            .entry(format!("{type_name}::{}", function.sig.ident))
            .or_default()
            .push(return_type);
    }
}

fn add_impl_associated_symbols(item_impl: &ItemImpl, counts: &mut BTreeMap<String, usize>) {
    if item_impl.trait_.is_some() {
        return;
    }
    let Some(type_name) = impl_type(item_impl) else {
        return;
    };
    for item in &item_impl.items {
        let ImplItem::Fn(function) = item else {
            continue;
        };
        *counts
            .entry(format!("{type_name}::{}", function.sig.ident))
            .or_insert(0usize) += 1;
    }
}

pub(super) fn file_free_functions(file: &syn::File) -> BTreeSet<String> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Fn(ItemFn { sig, .. }) => Some(sig.ident.to_string()),
            _ => None,
        })
        .collect()
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

fn explicit_return_type(signature: &syn::Signature, self_type: &str) -> Option<String> {
    let syn::ReturnType::Type(_, output) = &signature.output else {
        return None;
    };
    let syn::Type::Path(path) = output.as_ref() else {
        return None;
    };
    let segment = path.path.segments.last()?;
    (segment.ident == "Self")
        .then_some(self_type.to_string())
        .or_else(|| Some(segment.ident.to_string()))
}
