use std::path::Path;

use super::capability_manifest_harness::HarnessResolver;
use crate::{
    capability_manifest::{
        CapabilityManifestAudit, KleActualFrameHarness, KleActualInputEvidence, KleSourceLocator,
    },
    source_inventory_repo::SourceInventoryRepo,
};

pub(crate) struct KleEvidenceValidator;

impl KleEvidenceValidator {
    pub(crate) fn validate(evidence: &KleActualInputEvidence) -> Result<(), String> {
        let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .map_err(|error| format!("failed to resolve KLE workspace root: {error}"))?;
        Self::validate_at(&workspace_root, evidence)
    }

    pub(crate) fn validate_at(
        workspace_root: &Path,
        evidence: &KleActualInputEvidence,
    ) -> Result<(), String> {
        let mut failures = Vec::new();
        CapabilityManifestAudit::collect_failure(
            &mut failures,
            Self::validate_frame_harness(workspace_root, &evidence.harness),
        );
        let source = match read_actual_input_source(workspace_root, evidence, &mut failures) {
            Some(source) => source,
            None => return Err(failures.join("; ")),
        };
        let Some(test) = Self::find_test_function(&source, evidence.selector) else {
            add_missing_selector_failures(&mut failures, evidence);
            return Err(failures.join("; "));
        };
        CapabilityManifestAudit::collect_failure(
            &mut failures,
            HarnessResolver::resolve(workspace_root, evidence, test).map(|_| ()),
        );
        failures
            .is_empty()
            .then_some(())
            .ok_or_else(|| failures.join("; "))
    }

    pub(crate) fn find_test_function(source: &str, expected_function: &str) -> Option<syn::ItemFn> {
        syn::parse_file(source)
            .ok()?
            .items
            .into_iter()
            .find_map(|item| {
                let syn::Item::Fn(function) = item else {
                    return None;
                };
                (function.sig.ident == expected_function
                    && (function
                        .attrs
                        .iter()
                        .any(|attribute| attribute.path().is_ident("test"))
                        || expected_function == "run_frame"))
                    .then_some(function)
            })
    }

    pub(crate) fn validate_frame_harness(
        workspace_root: &Path,
        harness: &KleActualFrameHarness,
    ) -> Result<(), String> {
        let mut failures = Vec::new();
        for (locator, name) in [
            (
                harness.public_show_callsite,
                "actual-frame public show callsite",
            ),
            (
                harness.scenario_implementation,
                "actual-frame scenario implementation",
            ),
            (harness.raw_input_root, "RawInput root"),
            (harness.raw_input_construction, "RawInput construction"),
        ] {
            CapabilityManifestAudit::collect_failure(
                &mut failures,
                validate_kle_source_locator(workspace_root, &locator, name),
            );
        }
        failures
            .is_empty()
            .then_some(())
            .ok_or_else(|| failures.join("; "))
    }
}

fn read_actual_input_source(
    workspace_root: &Path,
    evidence: &KleActualInputEvidence,
    failures: &mut Vec<String>,
) -> Option<String> {
    SourceInventoryRepo::read_file(&workspace_root.join(evidence.source_path))
        .map_err(|_| {
            failures.push(format!(
                "missing KLE actual-input source: {}",
                evidence.source_path
            ));
            add_missing_selector_failures(failures, evidence);
        })
        .ok()
}

fn add_missing_selector_failures(failures: &mut Vec<String>, evidence: &KleActualInputEvidence) {
    failures.push(format!(
        "missing KLE actual-input selector: {}::{}",
        evidence.source_path, evidence.selector
    ));
    failures.push(format!(
        "missing KLE actual-input harness call: {}::{} requires {}",
        evidence.source_path, evidence.selector, evidence.harness.call_path
    ));
}

fn validate_kle_source_locator(
    workspace_root: &Path,
    locator: &KleSourceLocator,
    name: &str,
) -> Result<(), String> {
    let source = SourceInventoryRepo::read_file(&workspace_root.join(locator.source_path))
        .map_err(|_| format!("missing KLE {name} source: {}", locator.source_path))?;
    let Some(actual_line) = source.lines().nth(locator.line.saturating_sub(1)) else {
        return Err(format!(
            "missing KLE {name} line: {}:{}",
            locator.source_path, locator.line
        ));
    };
    actual_line
        .contains(locator.marker)
        .then_some(())
        .ok_or_else(|| {
            format!(
                "KLE {name} marker mismatch: {}:{} expected {:?}",
                locator.source_path, locator.line, locator.marker,
            )
        })
}
