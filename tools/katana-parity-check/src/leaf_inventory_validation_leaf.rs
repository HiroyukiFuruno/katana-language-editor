use super::super::leaf_inventory_evidence::{
    validate_leaf_host_e2e, validate_leaf_kle_evidence, validate_leaf_source,
};
use super::super::leaf_inventory_types::{
    LeafCapability, LeafEvidenceKind, MISSING_LEAF_SCHEMA_FIELD, OWNERS,
    REJECTED_KLE_EVIDENCE_KINDS,
};
use crate::capability_manifest::HostEffectKind;
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
pub(super) fn validate_leaf(
    leaf: &LeafCapability,
    katana_root: &Path,
    kle_root: &Path,
) -> Result<(), String> {
    let mut failures = Vec::new();
    collect_failure(&mut failures, validate_leaf_source(leaf, katana_root));
    collect_failure(&mut failures, validate_leaf_kle_evidence(leaf, kle_root));
    collect_failure(&mut failures, validate_leaf_host_e2e(leaf, kle_root));
    failures
        .is_empty()
        .then_some(())
        .ok_or_else(|| failures.join("; "))
}

pub(super) fn validate_leaf_declaration(leaf: &LeafCapability) -> Result<(), String> {
    validate_leaf_shape(leaf, &mut BTreeSet::<&'static str>::new())?;
    Ok(())
}

pub(super) fn validate_leaf_shape(
    leaf: &LeafCapability,
    ids: &mut BTreeSet<&'static str>,
) -> Result<(), String> {
    if !ids.insert(leaf.id) {
        return Err(format!(
            "duplicate exact editor leaf capability: {}",
            leaf.id
        ));
    }
    if leaf.parent_group.is_empty()
        || leaf.source.path.is_empty()
        || leaf.source.line == 0
        || leaf.source.marker.is_empty()
    {
        return Err(format!(
            "incomplete exact editor leaf declaration: {}",
            leaf.id
        ));
    }
    if leaf.owners != OWNERS {
        return Err(format!(
            "leaf {} is missing KUC/KLE/KatanA ownership",
            leaf.id
        ));
    }
    if leaf.katana_source_symbol.is_empty()
        || leaf.katana_source_symbol == MISSING_LEAF_SCHEMA_FIELD
    {
        return Err(format!(
            "leaf {} is missing a KatanA source symbol",
            leaf.id
        ));
    }
    if leaf.visible_menu_path.is_empty() || leaf.visible_menu_path == MISSING_LEAF_SCHEMA_FIELD {
        return Err(format!("leaf {} is missing a visible/menu path", leaf.id));
    }
    if leaf.state_condition.is_empty() || leaf.state_condition == MISSING_LEAF_SCHEMA_FIELD {
        return Err(format!("leaf {} is missing a state condition", leaf.id));
    }
    if leaf.typed_kle_request_or_state.is_empty()
        || leaf.typed_kle_request_or_state == MISSING_LEAF_SCHEMA_FIELD
    {
        return Err(format!(
            "leaf {} is missing typed KLE request/state",
            leaf.id
        ));
    }
    if leaf.expected_actual_katana_effect.is_empty()
        || leaf.expected_actual_katana_effect == MISSING_LEAF_SCHEMA_FIELD
    {
        return Err(format!(
            "leaf {} is missing expected actual KatanA effect",
            leaf.id
        ));
    }
    if leaf.host_e2e.effect == HostEffectKind::Missing {
        return Err(format!(
            "leaf {} has missing actual KatanA host-effect coverage",
            leaf.id
        ));
    }
    if leaf.expected_actual_katana_effect != leaf.host_e2e.effect.description() {
        return Err(format!(
            "leaf {} expected actual KatanA effect {} does not match host-e2e effect {}",
            leaf.id,
            leaf.expected_actual_katana_effect,
            leaf.host_e2e.effect.description()
        ));
    }
    if leaf.id == leaf.parent_group || !leaf.id.contains('.') {
        return Err(format!("leaf {} is a top-level group aggregate", leaf.id));
    }
    if !leaf.kle.selector.starts_with("public_show_")
        || leaf.kle.source_path.is_empty()
        || is_rejected_evidence(leaf.kle.kind)
    {
        return Err(format!("leaf {} has non-actual KLE evidence", leaf.id));
    }
    if leaf.host_e2e.test.target.is_empty()
        || leaf.host_e2e.test.source_path.is_empty()
        || leaf.host_e2e.test.selector.is_empty()
    {
        return Err(format!(
            "leaf {} does not declare exact KLE host E2E target and selector",
            leaf.id
        ));
    }
    Ok(())
}

fn is_rejected_evidence(kind: LeafEvidenceKind) -> bool {
    REJECTED_KLE_EVIDENCE_KINDS
        .iter()
        .any(|rejected| std::mem::discriminant(rejected) == std::mem::discriminant(&kind))
}

pub(super) fn kle_workspace_root() -> Result<PathBuf, String> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|error| format!("failed to resolve KLE workspace root: {error}"))
}

fn collect_failure(failures: &mut Vec<String>, result: Result<(), String>) {
    if let Err(error) = result {
        failures.push(error);
    }
}
