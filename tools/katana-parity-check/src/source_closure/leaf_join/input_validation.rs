use std::collections::HashSet;

use super::super::model::{
    ActionOriginsArtifact, BranchCatalogArtifact, ManifestRoot, SourceClosureArtifact,
};

pub(super) fn validate(
    source: &SourceClosureArtifact,
    branches: &BranchCatalogArtifact,
    actions: &ActionOriginsArtifact,
) -> Result<(), String> {
    ensure_same_root(&source.root, &branches.root, &actions.root)?;

    let mut source_paths = HashSet::new();
    for file in &source.files {
        if !source_paths.insert(&file.path) {
            return Err(format!(
                "source closure contains duplicate source path: {}",
                file.path
            ));
        }
    }

    let mut branch_ids = HashSet::new();
    for branch in &branches.branches {
        if !branch_ids.insert(&branch.branch_id) {
            return Err(format!(
                "branch catalog contains duplicate branch ID: {}",
                branch.branch_id
            ));
        }
        if !source_paths.contains(&branch.file) {
            return Err(format!(
                "branch references source path absent from source closure: {}",
                branch.file
            ));
        }
    }

    Ok(())
}

fn ensure_same_root(
    source: &ManifestRoot,
    branches: &ManifestRoot,
    actions: &ManifestRoot,
) -> Result<(), String> {
    if !same_root(source, branches) || !same_root(source, actions) {
        return Err("source-closure triple has inconsistent manifest roots".to_string());
    }
    Ok(())
}

fn same_root(left: &ManifestRoot, right: &ManifestRoot) -> bool {
    left.schema_version == right.schema_version
        && left.katana_revision == right.katana_revision
        && left.katana_tree_fingerprint == right.katana_tree_fingerprint
        && left.katana_external_ui_fingerprint == right.katana_external_ui_fingerprint
        && left.user_mandated_extensions_fingerprint == right.user_mandated_extensions_fingerprint
        && left.source_universe_fingerprint == right.source_universe_fingerprint
        && left.requirement_source_aliases_fingerprint
            == right.requirement_source_aliases_fingerprint
        && left.kle_tree_fingerprint == right.kle_tree_fingerprint
        && left.kuc_tree_fingerprint == right.kuc_tree_fingerprint
        && left.release_profile_matrix_fingerprint == right.release_profile_matrix_fingerprint
        && left.generator_fingerprint == right.generator_fingerprint
        && left.generated_at_utc == right.generated_at_utc
        && left.static_leaf_count == right.static_leaf_count
        && left.expected_leaf_ids == right.expected_leaf_ids
}
