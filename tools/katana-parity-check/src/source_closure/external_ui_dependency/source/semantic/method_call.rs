use std::collections::{BTreeMap, BTreeSet};

use syn::{Expr, ExprCall, ExprMethodCall, FnArg, Local, Pat, Signature, Type};

pub(super) fn parameter_types_with_definitions(
    signature: &Signature,
    defined_types: &BTreeSet<String>,
) -> BTreeMap<String, String> {
    signature
        .inputs
        .iter()
        .filter_map(|argument| match argument {
            FnArg::Typed(typed) => parameter_type(typed, defined_types),
            FnArg::Receiver(_) => None,
        })
        .collect()
}

pub(super) fn target(
    call: &ExprMethodCall,
    binding_types: &BTreeMap<String, String>,
    associated: &BTreeSet<String>,
    clone_types: &BTreeSet<String>,
) -> Option<String> {
    let type_name = receiver_type(call.receiver.as_ref(), binding_types, clone_types)?;
    let target = format!("{type_name}::{}", call.method);
    associated.contains(&target).then_some(target)
}

pub(super) fn local_binding_type(
    local: &Local,
    associated_constructor_types: &BTreeMap<String, String>,
    associated_option_return_types: &BTreeMap<String, String>,
    default_types: &BTreeSet<String>,
) -> Option<(String, String)> {
    let binding = local_binding_name(local)?;
    let expression = local.init.as_ref()?.expr.as_ref();
    direct_constructor_type(expression, associated_constructor_types)
        .or_else(|| option_default_type(expression, associated_option_return_types, default_types))
        .map(|type_name| (binding, type_name))
}

pub(super) fn local_binding_name(local: &Local) -> Option<String> {
    let Pat::Ident(binding) = &local.pat else {
        return None;
    };
    Some(binding.ident.to_string())
}

fn parameter_type(
    typed: &syn::PatType,
    defined_types: &BTreeSet<String>,
) -> Option<(String, String)> {
    let Pat::Ident(binding) = typed.pat.as_ref() else {
        return None;
    };
    let type_name = match typed.ty.as_ref() {
        Type::Path(type_path) if type_path.qself.is_none() => {
            direct_type_path_name(&type_path.path, defined_types)?
        }
        _ => return None,
    };
    Some((binding.ident.to_string(), type_name))
}

fn direct_type_path_name(path: &syn::Path, defined_types: &BTreeSet<String>) -> Option<String> {
    let segment = path.segments.last()?;
    let name = segment.ident.to_string();
    defined_types.contains(&name).then_some(name)
}

fn path_symbol(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

fn receiver_type(
    receiver: &Expr,
    binding_types: &BTreeMap<String, String>,
    clone_types: &BTreeSet<String>,
) -> Option<String> {
    match receiver {
        Expr::Path(path) => binding_types
            .get(&path.path.get_ident()?.to_string())
            .cloned(),
        Expr::MethodCall(call) if call.method == "clone" && call.args.is_empty() => {
            let type_name = receiver_type(call.receiver.as_ref(), binding_types, clone_types)?;
            clone_types.contains(&type_name).then_some(type_name)
        }
        _ => None,
    }
}

fn direct_constructor_type(
    expression: &Expr,
    associated_constructor_types: &BTreeMap<String, String>,
) -> Option<String> {
    let Expr::Call(ExprCall { func, .. }) = expression else {
        return None;
    };
    let Expr::Path(path) = func.as_ref() else {
        return None;
    };
    associated_constructor_types
        .get(&path_symbol(&path.path))
        .cloned()
}

fn option_default_type(
    expression: &Expr,
    associated_option_return_types: &BTreeMap<String, String>,
    default_types: &BTreeSet<String>,
) -> Option<String> {
    let Expr::MethodCall(call) = expression else {
        return None;
    };
    if call.method != "unwrap_or_default" || !call.args.is_empty() {
        return None;
    }
    let type_name =
        direct_option_constructor_type(call.receiver.as_ref(), associated_option_return_types)?;
    default_types.contains(&type_name).then_some(type_name)
}

fn direct_option_constructor_type(
    expression: &Expr,
    associated_option_return_types: &BTreeMap<String, String>,
) -> Option<String> {
    let Expr::Call(ExprCall { func, .. }) = expression else {
        return None;
    };
    let Expr::Path(path) = func.as_ref() else {
        return None;
    };
    associated_option_return_types
        .get(&path_symbol(&path.path))
        .cloned()
}
