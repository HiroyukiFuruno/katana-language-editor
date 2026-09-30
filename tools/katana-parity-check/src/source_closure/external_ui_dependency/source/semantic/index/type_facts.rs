use std::collections::{BTreeMap, BTreeSet};

use syn::{Attribute, GenericArgument, ImplItem, Item, ItemImpl, PathArguments, Type};

use super::impl_type;

pub(in super::super) fn associated_option_return_types(
    sources: &[(String, syn::File)],
) -> BTreeMap<String, String> {
    let mut candidates = BTreeMap::new();
    for (_, file) in sources {
        for item in &file.items {
            let Item::Impl(item_impl) = item else {
                continue;
            };
            add_impl_option_return_types(item_impl, &mut candidates);
        }
    }
    unique_candidates(candidates)
}

pub(in super::super) fn derived_types(
    sources: &[(String, syn::File)],
    trait_name: &str,
) -> BTreeSet<String> {
    let mut counts = BTreeMap::new();
    for (_, file) in sources {
        for item in &file.items {
            let Item::Struct(item_struct) = item else {
                continue;
            };
            if derives(&item_struct.attrs, trait_name) {
                *counts
                    .entry(item_struct.ident.to_string())
                    .or_insert(0usize) += 1;
            }
        }
    }
    counts
        .into_iter()
        .filter_map(|(type_name, count)| (count == 1).then_some(type_name))
        .collect()
}

fn add_impl_option_return_types(
    item_impl: &ItemImpl,
    candidates: &mut BTreeMap<String, Vec<String>>,
) {
    let Some(self_type) = impl_type(item_impl) else {
        return;
    };
    for item in &item_impl.items {
        let ImplItem::Fn(function) = item else {
            continue;
        };
        let Some(return_type) = option_return_type(&function.sig.output, &self_type) else {
            continue;
        };
        candidates
            .entry(format!("{self_type}::{}", function.sig.ident))
            .or_default()
            .push(return_type);
    }
}

fn unique_candidates(candidates: BTreeMap<String, Vec<String>>) -> BTreeMap<String, String> {
    candidates
        .into_iter()
        .filter_map(|(symbol, mut values)| (values.len() == 1).then(|| (symbol, values.remove(0))))
        .collect()
}

fn option_return_type(output: &syn::ReturnType, self_type: &str) -> Option<String> {
    let syn::ReturnType::Type(_, output) = output else {
        return None;
    };
    let Type::Path(option) = output.as_ref() else {
        return None;
    };
    let segment = option.path.segments.last()?;
    if segment.ident != "Option" {
        return None;
    }
    let PathArguments::AngleBracketed(arguments) = &segment.arguments else {
        return None;
    };
    let GenericArgument::Type(Type::Path(inner)) = arguments.args.first()? else {
        return None;
    };
    let inner = inner.path.segments.last()?.ident.to_string();
    Some(if inner == "Self" {
        self_type.into()
    } else {
        inner
    })
}

fn derives(attributes: &[Attribute], trait_name: &str) -> bool {
    attributes.iter().any(|attribute| {
        attribute.path().is_ident("derive")
            && attribute
                .parse_args_with(
                    syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
                )
                .is_ok_and(|paths| {
                    paths.iter().any(|path| {
                        path.segments
                            .last()
                            .is_some_and(|segment| segment.ident == trait_name)
                    })
                })
    })
}
