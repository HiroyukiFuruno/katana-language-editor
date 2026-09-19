use std::collections::BTreeSet;

use super::artifact_validator::ArtifactValidationMode;
use super::leaf_host_e2e_validation;
use super::model::{ActionOriginsArtifact, BranchCatalogArtifact, LeafManifestArtifact};
use super::validator_helpers::{
    duplicates, is_canonical_profile_set_list, is_placeholder, is_terminal_status,
    kle_mounted_in_katana,
};

pub(super) fn validate(
    actions: &ActionOriginsArtifact,
    branches: &BranchCatalogArtifact,
    manifest: &LeafManifestArtifact,
    mode: ArtifactValidationMode,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let leaf_ids = validate_leafs(manifest, mode, errors);
    validate_actions(actions, &leaf_ids, errors);
    validate_branches(branches, &leaf_ids, errors);
    leaf_ids
}

fn validate_leafs(
    manifest: &LeafManifestArtifact,
    mode: ArtifactValidationMode,
    errors: &mut Vec<String>,
) -> BTreeSet<String> {
    let mut unique = BTreeSet::new();
    for leaf in &manifest.leafs {
        if leaf.leaf_id.trim().is_empty() || is_placeholder(leaf.leaf_id.as_str()) {
            errors.push("leaf-manifest has placeholder leaf id".to_string());
        }
        if !is_canonical_profile_set_list(&leaf.required_profile_ids) {
            errors.push(format!(
                "leaf-manifest leaf {} has invalid/missing required profiles",
                leaf.leaf_id
            ));
        }
        if leaf.status.trim().is_empty()
            || is_placeholder(leaf.status.as_str())
            || !is_terminal_status(leaf.status.as_str())
            || is_unresolved(leaf.kuc_component.as_str())
            || is_placeholder(leaf.kle_opaque_transit.as_str())
            || is_placeholder(leaf.kle_public_show_signature.as_str())
            || is_placeholder(leaf.declared_effect_kind.as_str())
            || is_unresolved(leaf.storybook_stage_id.as_str())
        {
            errors.push(format!(
                "leaf-manifest leaf {} has placeholder or invalid status/fields",
                leaf.leaf_id
            ));
        }
        leaf_host_e2e_validation::validate(leaf, mode, errors);
        unique.insert(leaf.leaf_id.clone());
    }
    if unique.is_empty() {
        errors.push("leaf-manifest has no leaves".to_string());
    }
    if unique.len() != manifest.leafs.len() {
        errors.push("leaf-manifest has duplicate leaf ids".to_string());
    }
    unique
}

fn is_unresolved(value: &str) -> bool {
    value.trim_start().starts_with("unresolved:")
}

fn validate_actions(
    actions: &ActionOriginsArtifact,
    leaf_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if !actions.unresolved_action_origins.is_empty() {
        errors.push(format!(
            "action-origins contains unresolved action origins: {}",
            actions.unresolved_action_origins.len()
        ));
    }
    for action in &actions.actions {
        if action.action_id.trim().is_empty()
            || action.origin_classification.trim().is_empty()
            || action.kle_leafs.is_empty()
            || is_placeholder(action.action_id.as_str())
        {
            errors.push("action-origins has placeholder or missing action metadata".to_string());
        }
        if !is_canonical_profile_set_list(&action.active_profile_ids) {
            errors.push(format!(
                "action-origins action {} has invalid/missing profile list",
                action.action_id
            ));
        }
        for leaf_id in &action.kle_leafs {
            if !leaf_ids.contains(leaf_id) {
                errors.push(format!(
                    "action-origins action {} references unknown leaf {}",
                    action.action_id, leaf_id
                ));
            }
        }
        if action
            .construction_sites
            .iter()
            .any(|site| is_placeholder(site.as_str()))
        {
            errors.push(format!(
                "action-origins action {} has placeholder construction site",
                action.action_id
            ));
        }
    }
}

fn validate_branches(
    branches: &BranchCatalogArtifact,
    leaf_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    if !branches.unclassified_branch_ids.is_empty() {
        errors.push(format!(
            "branch-catalog contains unclassified branches: {}",
            branches.unclassified_branch_ids.len()
        ));
    }
    if branches.branches.is_empty() {
        errors.push("branch-catalog has no branch entries".to_string());
    }
    if duplicates(branches.branches.iter().map(|branch| &branch.branch_id)) {
        errors.push("branch-catalog contains duplicate branch ids".to_string());
    }
    for branch in &branches.branches {
        if branch.branch_id.trim().is_empty()
            || branch.file.trim().is_empty()
            || branch.symbol.trim().is_empty()
            || is_placeholder(branch.branch_id.as_str())
            || is_placeholder(branch.file.as_str())
            || is_placeholder(branch.symbol.as_str())
        {
            errors.push(format!(
                "branch-catalog branch {} has placeholder fields",
                branch.branch_id
            ));
        }
        if !is_canonical_profile_set_list(&branch.active_profile_ids) {
            errors.push(format!(
                "branch-catalog branch {} has invalid/missing active profiles",
                branch.branch_id
            ));
        }
        if kle_mounted_in_katana(branch.file.as_str()) {
            errors
                .push("branch-catalog asserts KLE mounted inside KatanA source files".to_string());
        }
        if is_placeholder(branch.source_excerpt_sha256.as_str())
            || branch.span.start_line == 0
            || branch.span.end_line == 0
            || branch.span.end_line < branch.span.start_line
        {
            errors.push(format!(
                "branch-catalog branch {} has placeholder or invalid span",
                branch.branch_id
            ));
        }
        for outcome in &branch.outcomes {
            if let Some(leaf_id) = outcome.strip_prefix("leaf:")
                && !leaf_ids.contains(leaf_id)
            {
                errors.push(format!(
                    "branch-catalog branch {} references unknown leaf outcome {}",
                    branch.branch_id, leaf_id
                ));
            }
        }
    }
}
