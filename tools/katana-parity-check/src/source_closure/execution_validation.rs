use std::collections::BTreeSet;

use super::artifact_validator::ArtifactValidationMode;
use super::execution_coverage;
use super::model::{ExecutionRecordArtifact, LeafManifestArtifact, StorybookArtifacts};
use super::validator_helpers::{
    duplicates, is_canonical_profile_set, is_placeholder, is_terminal_status,
};

pub(super) fn validate(
    executions: &ExecutionRecordArtifact,
    storybook: &StorybookArtifacts,
    leaf_manifest: &LeafManifestArtifact,
    leaf_ids: &BTreeSet<String>,
    mode: ArtifactValidationMode,
    errors: &mut Vec<String>,
) {
    if !executions.stale_or_missing_leafs.is_empty() {
        errors.push(format!(
            "execution-record has stale or missing leaves: {:?}",
            executions.stale_or_missing_leafs
        ));
    }
    if duplicates(
        executions
            .executions
            .iter()
            .map(|execution| &execution.execution_id),
    ) {
        errors.push("execution-record has duplicate execution ids".to_string());
    }
    if mode == ArtifactValidationMode::Full && executions.executions.is_empty() {
        errors.push("execution-record has no execution entries".to_string());
    }
    for execution in &executions.executions {
        if execution.execution_id.trim().is_empty()
            || is_placeholder(&execution.execution_id)
            || is_placeholder(&execution.leaf_id)
            || is_placeholder(&execution.action_route_signature)
            || is_placeholder(&execution.status)
            || execution.status.trim().is_empty()
            || !is_terminal_status(&execution.status)
        {
            errors.push(format!(
                "execution-record execution {} has placeholder or invalid fields",
                execution.execution_id
            ));
        }
        if !is_canonical_profile_set(&execution.execution_profile_id) {
            errors.push(format!(
                "execution-record execution {} has invalid profile",
                execution.execution_id
            ));
        }
        if !leaf_ids.contains(&execution.leaf_id) {
            errors.push(format!(
                "execution-record references unknown leaf {}",
                execution.leaf_id
            ));
        }
    }
    match mode {
        ArtifactValidationMode::Rc | ArtifactValidationMode::KleRelease => {
            execution_coverage::validate_rc_boundary(executions, errors)
        }
        ArtifactValidationMode::Full => {
            execution_coverage::validate_full(executions, leaf_manifest, leaf_ids, errors)
        }
    }

    if duplicates(
        storybook
            .artifacts
            .iter()
            .map(|artifact| &artifact.story_id),
    ) {
        errors.push("storybook-artifacts has duplicate story ids".to_string());
    }
    if storybook.artifacts.is_empty() {
        errors.push("storybook-artifacts has no entries".to_string());
    }
    for artifact in &storybook.artifacts {
        if artifact.story_id.trim().is_empty()
            || artifact.leaf_id.trim().is_empty()
            || artifact.stage_id.trim().is_empty()
            || artifact.frame_record_id.trim().is_empty()
            || artifact.media_sha256.trim().is_empty()
        {
            errors.push(format!(
                "storybook-artifacts has invalid story {}",
                artifact.story_id
            ));
        }
        if is_placeholder(artifact.media_sha256.as_str()) {
            errors.push(format!(
                "storybook-artifacts has placeholder media hash {}",
                artifact.story_id
            ));
        }
        if !is_canonical_profile_set(&artifact.profile_id) {
            errors.push(format!(
                "storybook-artifacts story {} has invalid profile",
                artifact.story_id
            ));
        }
        if !leaf_ids.contains(&artifact.leaf_id) {
            errors.push(format!(
                "storybook-artifacts story {} references unknown leaf {}",
                artifact.story_id, artifact.leaf_id
            ));
        }
    }
    validate_storybook_coverage(storybook, leaf_manifest, leaf_ids, errors);
}

fn validate_storybook_coverage(
    storybook: &StorybookArtifacts,
    leaf_manifest: &LeafManifestArtifact,
    leaf_ids: &BTreeSet<String>,
    errors: &mut Vec<String>,
) {
    let covered_leaf_ids = storybook
        .artifacts
        .iter()
        .map(|artifact| artifact.leaf_id.as_str())
        .collect::<BTreeSet<_>>();
    for leaf_id in leaf_ids {
        if !covered_leaf_ids.contains(leaf_id.as_str()) {
            errors.push(format!(
                "storybook-artifacts is missing required leaf artifact: {leaf_id}"
            ));
        }
    }
    for leaf in &leaf_manifest.leafs {
        if !storybook.artifacts.iter().any(|artifact| {
            artifact.leaf_id == leaf.leaf_id && artifact.stage_id == leaf.storybook_stage_id
        }) {
            errors.push(format!(
                "storybook-artifacts is missing declared stage {} for leaf {}",
                leaf.storybook_stage_id, leaf.leaf_id
            ));
        }
        let covered_profiles = storybook
            .artifacts
            .iter()
            .filter(|artifact| artifact.leaf_id == leaf.leaf_id)
            .map(|artifact| artifact.profile_id.as_str())
            .collect::<BTreeSet<_>>();
        for profile_id in &leaf.required_profile_ids {
            if !covered_profiles.contains(profile_id.as_str()) {
                errors.push(format!(
                    "storybook-artifacts is missing required profile {profile_id} for leaf {}",
                    leaf.leaf_id
                ));
            }
        }
    }
}
