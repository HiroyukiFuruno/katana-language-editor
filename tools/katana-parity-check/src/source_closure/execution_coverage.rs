use std::collections::BTreeSet;

use super::model::{ExecutionRecordArtifact, LeafManifestArtifact, LeafStatusRecord};

pub(super) fn validate_rc_boundary(executions: &ExecutionRecordArtifact, errors: &mut Vec<String>) {
    if !executions.executions.is_empty() {
        errors.push(
            "RC source-closure artifacts must not claim KatanA host execution before #336 adoption"
                .to_string(),
        );
    }
}

pub(super) fn validate_full(
    executions: &ExecutionRecordArtifact,
    leaf_manifest: &LeafManifestArtifact,
    leaf_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let covered_leaf_ids = executions
        .executions
        .iter()
        .map(|execution| execution.leaf_id.as_str())
        .collect::<BTreeSet<_>>();
    for leaf_id in leaf_ids {
        if !covered_leaf_ids.contains(leaf_id.as_str()) {
            errors.push(format!(
                "execution-record is missing required leaf execution: {leaf_id}"
            ));
        }
    }
    for leaf in &leaf_manifest.leafs {
        validate_declared_execution(executions, leaf, errors);
        validate_leaf_profiles(executions, leaf, errors);
    }
}

fn validate_declared_execution(
    executions: &ExecutionRecordArtifact,
    leaf: &LeafStatusRecord,
    errors: &mut Vec<String>,
) {
    if !executions.executions.iter().any(|execution| {
        execution.leaf_id == leaf.leaf_id
            && leaf.execution_id.as_deref() == Some(execution.execution_id.as_str())
    }) {
        errors.push(format!(
            "execution-record is missing declared execution {:?} for leaf {}",
            leaf.execution_id, leaf.leaf_id
        ));
    }
}

fn validate_leaf_profiles(
    executions: &ExecutionRecordArtifact,
    leaf: &LeafStatusRecord,
    errors: &mut Vec<String>,
) {
    let covered_profiles = executions
        .executions
        .iter()
        .filter(|execution| execution.leaf_id == leaf.leaf_id)
        .map(|execution| execution.execution_profile_id.as_str())
        .collect::<BTreeSet<_>>();
    for profile_id in &leaf.required_profile_ids {
        if !covered_profiles.contains(profile_id.as_str()) {
            errors.push(format!(
                "execution-record is missing required profile {profile_id} for leaf {}",
                leaf.leaf_id
            ));
        }
    }
}
