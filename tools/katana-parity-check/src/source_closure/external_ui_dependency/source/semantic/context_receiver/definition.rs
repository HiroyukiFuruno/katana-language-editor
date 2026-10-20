use std::collections::{BTreeMap, BTreeSet};

use syn::{FnArg, ImplItem, Item, ItemImpl, Pat, ReceiverKind, Type};

pub(super) fn context_methods(sources: &[(String, syn::File)]) -> Option<BTreeSet<String>> {
    has_one_plain_struct(sources, "Context").then(|| {
        sources
            .iter()
            .flat_map(|(_, file)| context_method_facts(file))
            .fold(BTreeMap::new(), merge_method_facts)
            .into_iter()
            .filter_map(|(name, (count, valid))| (count == 1 && valid).then_some(name))
            .collect()
    })
}

pub(super) fn ui_ctx(sources: &[(String, syn::File)]) -> Option<String> {
    if !has_one_plain_struct(sources, "Ui") {
        return None;
    }
    let (count, valid) = sources
        .iter()
        .flat_map(|(_, file)| ui_ctx_facts(file))
        .fold((0, false), |(count, _), valid| (count + 1, valid));
    (count == 1 && valid).then_some("ctx".into())
}

pub(super) fn ui_binding(signature: &syn::Signature) -> Option<String> {
    signature.inputs.iter().find_map(|argument| {
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
        is_plain_type(reference.elem.as_ref(), "Ui").then(|| binding.ident.to_string())
    })
}

fn context_method_facts(file: &syn::File) -> Vec<(String, bool)> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) if is_plain_impl_type(item, "Context") => Some(item),
            _ => None,
        })
        .flat_map(|item| {
            item.items.iter().filter_map(move |member| {
                let ImplItem::Fn(function) = member else {
                    return None;
                };
                Some((
                    function.sig.ident.to_string(),
                    item.trait_.is_none()
                        && item.generics.params.is_empty()
                        && item.generics.where_clause.is_none()
                        && function.sig.generics.params.is_empty()
                        && function.sig.generics.where_clause.is_none()
                        && has_reference_receiver(&function.sig),
                ))
            })
        })
        .collect()
}

fn merge_method_facts(
    mut methods: BTreeMap<String, (usize, bool)>,
    (name, valid): (String, bool),
) -> BTreeMap<String, (usize, bool)> {
    let entry = methods.entry(name).or_default();
    entry.0 += 1;
    entry.1 = valid;
    methods
}

fn ui_ctx_facts(file: &syn::File) -> Vec<bool> {
    file.items
        .iter()
        .filter_map(|item| match item {
            Item::Impl(item) if is_plain_impl_type(item, "Ui") => Some(item),
            _ => None,
        })
        .flat_map(|item| {
            item.items.iter().filter_map(move |member| {
                let ImplItem::Fn(function) = member else {
                    return None;
                };
                (function.sig.ident == "ctx").then(|| {
                    item.trait_.is_none()
                        && item.generics.params.is_empty()
                        && item.generics.where_clause.is_none()
                        && function.sig.generics.params.is_empty()
                        && function.sig.generics.where_clause.is_none()
                        && has_shared_receiver(&function.sig)
                        && returns_context_reference(&function.sig)
                })
            })
        })
        .collect()
}

fn has_one_plain_struct(sources: &[(String, syn::File)], name: &str) -> bool {
    sources
        .iter()
        .flat_map(|(_, file)| file.items.iter())
        .try_fold(0, |count, item| match item {
            Item::Struct(item) if item.ident == name && item.generics.params.is_empty() => {
                Some(count + 1)
            }
            Item::Struct(item) if item.ident == name => None,
            Item::Enum(item) if item.ident == name => None,
            Item::Union(item) if item.ident == name => None,
            Item::Trait(item) if item.ident == name => None,
            Item::Type(item) if item.ident == name => None,
            _ => Some(count),
        })
        == Some(1)
}

fn is_plain_impl_type(item: &ItemImpl, type_name: &str) -> bool {
    is_plain_type(&item.self_ty, type_name)
}

fn is_plain_type(ty: &Type, type_name: &str) -> bool {
    matches!(ty, Type::Path(path)
        if path.qself.is_none()
            && path.path.leading_colon.is_none()
            && path.path.segments.len() == 1
            && path.path.segments[0].ident == type_name)
}

fn has_shared_receiver(signature: &syn::Signature) -> bool {
    signature.inputs.len() == 1 && has_reference_receiver(signature)
}

fn has_reference_receiver(signature: &syn::Signature) -> bool {
    signature.receiver().is_some_and(|receiver| {
        receiver.mutability.is_none() && matches!(receiver.kind, ReceiverKind::Reference(..))
    })
}

fn returns_context_reference(signature: &syn::Signature) -> bool {
    let syn::ReturnType::Type(_, output) = &signature.output else {
        return false;
    };
    matches!(output.as_ref(), Type::Reference(reference) if is_plain_type(&reference.elem, "Context"))
}
