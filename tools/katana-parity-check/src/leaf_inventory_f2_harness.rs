use std::{collections::BTreeSet, path::Path};

use crate::{
    capability_manifest::{AssociatedCallAnalysis, KleActualFrameHarness, KleSourceLocator},
    source_inventory_repo::SourceInventoryRepo,
};

const F2_SCENARIO: &str = "F2CommandChromeScenario";
const F2_HARNESS: &str = "F2RawInputHarness";

pub(super) fn validate_f2_scenario_raw_input_path(
    kle_root: &Path,
    harness: &KleActualFrameHarness,
) -> Result<(), String> {
    if harness.symbol != F2_SCENARIO {
        return Ok(());
    }
    let scenario = read_scenario(kle_root, &harness.scenario_implementation)?;
    let run_method = configured_method(harness)?;
    scenario_reaches_f2_raw_input(&scenario, run_method)
        .then_some(())
        .ok_or_else(|| {
            format!(
                "F2 scenario {} does not reach {}::show through its configured run path",
                harness.call_path, F2_HARNESS
            )
        })
}

fn read_scenario(kle_root: &Path, locator: &KleSourceLocator) -> Result<syn::ItemImpl, String> {
    let source =
        SourceInventoryRepo::read_file(&kle_root.join(locator.source_path)).map_err(|_| {
            format!(
                "missing F2 command-chrome scenario source: {}",
                locator.source_path
            )
        })?;
    let file = syn::parse_file(&source).map_err(|_| {
        format!(
            "failed to parse F2 command-chrome scenario: {}",
            locator.source_path
        )
    })?;
    file.items
        .into_iter()
        .find_map(|item| match item {
            syn::Item::Impl(implementation) if is_f2_scenario(&implementation) => {
                Some(implementation)
            }
            _ => None,
        })
        .ok_or_else(|| format!("missing {F2_SCENARIO} implementation"))
}

fn configured_method(harness: &KleActualFrameHarness) -> Result<&str, String> {
    let mut parts = harness.call_path.split("::");
    match (parts.next(), parts.next(), parts.next()) {
        (Some(F2_SCENARIO), Some(method), None) if !method.is_empty() => Ok(method),
        _ => Err(format!(
            "F2 command-chrome call path must be {F2_SCENARIO}::<method>: {}",
            harness.call_path
        )),
    }
}

fn scenario_reaches_f2_raw_input(scenario: &syn::ItemImpl, run_method: &str) -> bool {
    reaches_f2_raw_input(scenario, run_method, &mut BTreeSet::new())
}

fn reaches_f2_raw_input(
    scenario: &syn::ItemImpl,
    method: &str,
    visited: &mut BTreeSet<String>,
) -> bool {
    if !visited.insert(method.to_string()) {
        return false;
    }
    let Some(function) = find_method(scenario, method) else {
        return false;
    };
    if AssociatedCallAnalysis::collect(&function.block, F2_HARNESS, false)
        .iter()
        .any(|call| call == "show")
    {
        return true;
    }
    AssociatedCallAnalysis::collect(&function.block, F2_SCENARIO, true)
        .into_iter()
        .any(|next| reaches_f2_raw_input(scenario, &next, visited))
}

fn find_method<'a>(scenario: &'a syn::ItemImpl, method: &str) -> Option<&'a syn::ImplItemFn> {
    scenario.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(function) if function.sig.ident == method => Some(function),
        _ => None,
    })
}

fn is_f2_scenario(implementation: &syn::ItemImpl) -> bool {
    let syn::Type::Path(path) = implementation.self_ty.as_ref() else {
        return false;
    };
    path.qself.is_none()
        && path.path.segments.len() == 1
        && path.path.segments[0].ident == F2_SCENARIO
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_f2_scenario_path_to_raw_input_harness() -> Result<(), String> {
        let scenario = parse_scenario(
            "impl F2CommandChromeScenario {\nfn run_command_chrome() { Self::configure(); }\nfn configure() { F2RawInputHarness::show(); }\n}",
        )?;

        assert!(scenario_reaches_f2_raw_input(
            &scenario,
            "run_command_chrome"
        ));
        Ok(())
    }

    #[test]
    fn rejects_f2_scenario_without_raw_input_harness_path() -> Result<(), String> {
        let scenario = parse_scenario(
            "impl F2CommandChromeScenario {\nfn run_command_chrome() { Self::configure(); }\nfn configure() {}\n}",
        )?;

        assert!(!scenario_reaches_f2_raw_input(
            &scenario,
            "run_command_chrome"
        ));
        Ok(())
    }

    fn parse_scenario(source: &str) -> Result<syn::ItemImpl, String> {
        syn::parse_file(source)
            .map_err(|error| format!("fixture parses: {error}"))?
            .items
            .into_iter()
            .find_map(|item| match item {
                syn::Item::Impl(implementation) => Some(implementation),
                _ => None,
            })
            .ok_or_else(|| "fixture must contain an impl".to_string())
    }
}
