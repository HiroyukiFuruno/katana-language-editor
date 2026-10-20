use std::collections::BTreeSet;

use super::capability_manifest_call_analysis::AssociatedCallAnalysis;
use crate::capability_manifest::KleActualInputEvidence;

pub(super) fn resolve_root_path(
    implementation: &syn::ItemImpl,
    evidence: &KleActualInputEvidence,
    roots: Vec<String>,
    run_method: &str,
) -> Result<Vec<String>, String> {
    let mut unresolved = Vec::new();
    for root in roots {
        if !scenario_has_method(implementation, &root) {
            unresolved.push(format!(
                "{}::{} is undefined",
                evidence.harness.symbol, root
            ));
        } else if let Some(path) = scenario_path_to_run(
            implementation,
            evidence.harness.symbol,
            &root,
            run_method,
            &mut BTreeSet::new(),
        ) {
            return Ok(path);
        } else {
            unresolved.push(format!(
                "{}::{} does not reach {}",
                evidence.harness.symbol, root, evidence.harness.call_path
            ));
        }
    }
    Err(format!(
        "missing KLE actual-input transitive harness evidence: {}::{} ExprCall cannot reach {} through {} ({})",
        evidence.source_path,
        evidence.selector,
        evidence.harness.call_path,
        evidence.harness.scenario_implementation.source_path,
        unresolved.join(", ")
    ))
}

pub(super) fn find_scenario_implementation(source: &str, symbol: &str) -> Option<syn::ItemImpl> {
    syn::parse_file(source)
        .ok()?
        .items
        .into_iter()
        .find_map(|item| {
            let syn::Item::Impl(implementation) = item else {
                return None;
            };
            let syn::Type::Path(path) = implementation.self_ty.as_ref() else {
                return None;
            };
            (path.qself.is_none()
                && path.path.segments.len() == 1
                && path.path.segments[0].ident == symbol)
                .then_some(implementation)
        })
}

fn scenario_path_to_run(
    implementation: &syn::ItemImpl,
    symbol: &str,
    method: &str,
    run_method: &str,
    visited: &mut BTreeSet<String>,
) -> Option<Vec<String>> {
    if !visited.insert(method.to_string()) {
        return None;
    }
    let function = find_method(implementation, method)?;
    if method == run_method {
        return Some(vec![method.to_string()]);
    }
    AssociatedCallAnalysis::collect(&function.block, symbol, true)
        .into_iter()
        .find_map(|callee| {
            let mut path =
                scenario_path_to_run(implementation, symbol, &callee, run_method, visited)?;
            path.insert(0, method.to_string());
            Some(path)
        })
}

fn find_method<'a>(implementation: &'a syn::ItemImpl, method: &str) -> Option<&'a syn::ImplItemFn> {
    implementation.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(function) if function.sig.ident == method => Some(function),
        _ => None,
    })
}

pub(super) fn scenario_has_method(implementation: &syn::ItemImpl, method: &str) -> bool {
    find_method(implementation, method).is_some()
}
