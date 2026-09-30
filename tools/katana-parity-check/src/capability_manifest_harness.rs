use std::path::Path;

use super::capability_manifest_call_analysis::AssociatedCallAnalysis;
use super::capability_manifest_harness_method_chain as method_chain;
use crate::{
    capability_manifest::KleActualInputEvidence, source_inventory_repo::SourceInventoryRepo,
};

pub(crate) struct HarnessResolver;

impl HarnessResolver {
    pub(crate) fn resolve(
        workspace_root: &Path,
        evidence: &KleActualInputEvidence,
        selector: syn::ItemFn,
    ) -> Result<Vec<String>, String> {
        let roots =
            AssociatedCallAnalysis::collect(&selector.block, evidence.harness.symbol, false);
        let run_method = Self::configured_run_method(evidence)?;
        let implementation = Self::read_implementation(workspace_root, evidence)?;
        if roots.is_empty() {
            let public_root_calls = AssociatedCallAnalysis::collect_receiver_method_calls(
                &selector.block,
                "host",
                run_method,
            );
            if public_root_calls.is_empty() {
                return Err(Self::missing_root_call(evidence));
            }
            if !Self::has_typed_receiver(&selector, "host", evidence.harness.symbol) {
                return Err(format!(
                    "KLE actual-input harness receiver host is not typed as {}",
                    evidence.harness.symbol
                ));
            }
            if !method_chain::scenario_has_method(&implementation, run_method) {
                return Err(format!(
                    "missing KLE public root method: {}",
                    evidence.harness.call_path
                ));
            }
            return Ok(public_root_calls);
        }
        method_chain::resolve_root_path(&implementation, evidence, roots, run_method)
    }

    fn has_typed_receiver(selector: &syn::ItemFn, receiver: &str, expected_type: &str) -> bool {
        selector.sig.inputs.iter().any(|argument| {
            let syn::FnArg::Typed(argument) = argument else {
                return false;
            };
            let syn::Pat::Ident(pattern) = argument.pat.as_ref() else {
                return false;
            };
            if pattern.ident != receiver {
                return false;
            }
            let mut type_path = argument.ty.as_ref();
            while let syn::Type::Reference(reference) = type_path {
                type_path = reference.elem.as_ref();
            }
            matches!(type_path, syn::Type::Path(path)
                if path.qself.is_none()
                    && path.path.is_ident(expected_type))
        })
    }

    pub(crate) fn format_chain(symbol: &str, chain: &[String]) -> String {
        chain
            .iter()
            .map(|method| format!("{symbol}::{method}"))
            .collect::<Vec<_>>()
            .join(" -> ")
    }

    fn configured_run_method(evidence: &KleActualInputEvidence) -> Result<&str, String> {
        let mut parts = evidence.harness.call_path.split("::");
        match (parts.next(), parts.next(), parts.next()) {
            (Some(symbol), Some(method), None)
                if symbol == evidence.harness.symbol && !method.is_empty() =>
            {
                Ok(method)
            }
            _ => Err(format!(
                "KLE actual-frame harness call path must be {}::<method>: {}",
                evidence.harness.symbol, evidence.harness.call_path
            )),
        }
    }

    fn missing_root_call(evidence: &KleActualInputEvidence) -> String {
        format!(
            "missing KLE actual-input harness call: {}::{} has no {}::<method> ExprCall that reaches {}",
            evidence.source_path,
            evidence.selector,
            evidence.harness.symbol,
            evidence.harness.call_path
        )
    }

    fn read_implementation(
        workspace_root: &Path,
        evidence: &KleActualInputEvidence,
    ) -> Result<syn::ItemImpl, String> {
        let locator = &evidence.harness.scenario_implementation;
        let source = SourceInventoryRepo::read_file(&workspace_root.join(locator.source_path))
            .map_err(|_| {
                format!(
                    "missing KLE actual-frame scenario implementation source: {}",
                    locator.source_path
                )
            })?;
        method_chain::find_scenario_implementation(&source, evidence.harness.symbol).ok_or_else(
            || {
                format!(
                    "missing KLE actual-frame scenario implementation: {} in {}",
                    evidence.harness.symbol, locator.source_path
                )
            },
        )
    }
}
