use std::path::Path;

use crate::{
    capability_manifest::{CapabilityManifestAudit, KleActualFrameHarness, KleActualInputEvidence},
    source_inventory_repo::SourceInventoryRepo,
};

use super::{
    leaf_inventory_f2_harness::validate_f2_scenario_raw_input_path,
    leaf_inventory_syntax::{ExactLeafAssertionCalls, LeafCaseRequirements, find_test_function},
    leaf_inventory_types::LeafCapability,
};

pub(super) fn validate_leaf_source(
    leaf: &LeafCapability,
    katana_root: &Path,
) -> Result<(), String> {
    let source = SourceInventoryRepo::read_file(&katana_root.join(leaf.source.path))
        .map_err(|_| format!("missing leaf KatanA source: {}", leaf.source.path))?;
    let line = source.lines().nth(leaf.source.line - 1).ok_or_else(|| {
        format!(
            "missing leaf KatanA source line: {}:{}",
            leaf.source.path, leaf.source.line
        )
    })?;
    if line.contains(leaf.source.marker) {
        Ok(())
    } else {
        Err(format!(
            "leaf source marker mismatch: {}:{} expected {:?}",
            leaf.source.path, leaf.source.line, leaf.source.marker
        ))
    }
}

pub(super) fn validate_leaf_kle_evidence(
    leaf: &LeafCapability,
    kle_root: &Path,
) -> Result<(), String> {
    let evidence = KleActualInputEvidence {
        feature_id: leaf.id,
        source_path: leaf.kle.source_path,
        selector: leaf.kle.selector,
        harness: leaf.kle.harness,
    };
    let mut failures = Vec::new();
    collect_failure(
        &mut failures,
        CapabilityManifestAudit::validate_actual_input_at(kle_root, &evidence).map_err(|error| {
            format!(
                "KLE leaf selector does not reach public EguiLanguageEditor::show through RawInput: {error}"
            )
        }),
    );
    collect_failure(
        &mut failures,
        validate_leaf_public_show_raw_input_contract(kle_root, &evidence.harness),
    );
    collect_failure(
        &mut failures,
        validate_f2_scenario_raw_input_path(kle_root, &evidence.harness),
    );
    collect_failure(&mut failures, validate_leaf_case_assertions(kle_root, leaf));
    failures
        .is_empty()
        .then_some(())
        .ok_or_else(|| failures.join("; "))
}

pub(super) fn validate_leaf_host_e2e(leaf: &LeafCapability, kle_root: &Path) -> Result<(), String> {
    CapabilityManifestAudit::validate_host_e2e_at(kle_root, &leaf.host_e2e).map_err(|error| {
        format!(
            "KLE leaf lacks actual KatanaHost request/UI-frame/document-effect evidence: {error}"
        )
    })
}

fn validate_leaf_public_show_raw_input_contract(
    kle_root: &Path,
    harness: &KleActualFrameHarness,
) -> Result<(), String> {
    let source =
        SourceInventoryRepo::read_file(&kle_root.join(harness.public_show_callsite.source_path))
            .map_err(|_| {
                format!(
                    "missing KLE public show RawInput harness source: {}",
                    harness.public_show_callsite.source_path
                )
            })?;
    let line = source
        .lines()
        .nth(harness.public_show_callsite.line.saturating_sub(1))
        .ok_or_else(|| {
            format!(
                "missing KLE public show RawInput harness line: {}:{}",
                harness.public_show_callsite.source_path, harness.public_show_callsite.line
            )
        })?;
    if !line.contains("editor.show(ui)") {
        return Err("KLE public show RawInput harness does not invoke editor.show(ui)".to_string());
    }
    if !source.contains("EguiLanguageEditor") || !source.contains("editor: &mut EguiLanguageEditor")
    {
        return Err(
            "KLE public show RawInput harness does not type editor as EguiLanguageEditor"
                .to_string(),
        );
    }
    Ok(())
}

fn validate_leaf_case_assertions(leaf_root: &Path, leaf: &LeafCapability) -> Result<(), String> {
    let source =
        SourceInventoryRepo::read_file(&leaf_root.join(leaf.kle.source_path)).map_err(|_| {
            format!(
                "missing KLE leaf actual-input source: {}",
                leaf.kle.source_path
            )
        })?;
    let test = find_test_function(&source, leaf.kle.selector).ok_or_else(|| {
        format!(
            "missing KLE leaf actual-input test selector: {}::{}",
            leaf.kle.source_path, leaf.kle.selector
        )
    })?;
    let requirements = LeafCaseRequirements::for_parent(leaf.parent_group)?;
    let calls = ExactLeafAssertionCalls::collect(&test.block);
    for assertion in [
        requirements.leaf_assertion,
        requirements.artifact_assertion,
        requirements.accesskit_assertion,
    ] {
        if !calls.contains(assertion, leaf.id) {
            return Err(format!(
                "KLE leaf selector {} does not enumerate {} via {}({:?})",
                leaf.kle.selector, leaf.id, assertion, leaf.id
            ));
        }
    }
    Ok(())
}

fn collect_failure(failures: &mut Vec<String>, result: Result<(), String>) {
    if let Err(error) = result {
        failures.push(error);
    }
}
