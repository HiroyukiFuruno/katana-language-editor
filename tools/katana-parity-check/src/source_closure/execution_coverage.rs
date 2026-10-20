use std::collections::BTreeSet;

use super::model::{
    ExecutionKind, ExecutionRecordArtifact, LeafManifestArtifact, LeafStatusRecord,
};

pub(super) fn validate_kle_boundary(
    executions: &ExecutionRecordArtifact,
    manifest: &LeafManifestArtifact,
    errors: &mut Vec<String>,
) {
    if executions
        .executions
        .iter()
        .any(|record| !matches!(record.execution_kind, ExecutionKind::KucConsumerArtifact))
    {
        errors
            .push("KLE release execution records must contain only KUC consumer artifacts".into());
    }
    for leaf in &manifest.leafs {
        let profiles = executions
            .executions
            .iter()
            .filter(|record| {
                record.leaf_id == leaf.leaf_id
                    && matches!(record.execution_kind, ExecutionKind::KucConsumerArtifact)
            })
            .map(|record| record.execution_profile_id.as_str())
            .collect::<Vec<_>>();
        let unique = profiles.iter().copied().collect::<BTreeSet<_>>();
        let expected = leaf
            .required_profile_ids
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if profiles.len() != unique.len() || unique != expected {
            errors.push(format!(
                "KLE consumer executions must cover each leaf/profile exactly once: {}",
                leaf.leaf_id
            ));
        }
    }
}

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
        .filter(|execution| matches!(execution.execution_kind, ExecutionKind::KatanaHost))
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
            && matches!(execution.execution_kind, ExecutionKind::KatanaHost)
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
        .filter(|execution| {
            execution.leaf_id == leaf.leaf_id
                && matches!(execution.execution_kind, ExecutionKind::KatanaHost)
        })
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
