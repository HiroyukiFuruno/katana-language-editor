use std::collections::{BTreeMap, BTreeSet};

use super::artifact_paths::ArtifactPaths;
use super::fingerprint::sha256_hex;
use super::kle_release_evidence_integrity::{
    ExpectedKleReleaseEvidence, parse_and_validate_integrity, validate_kle_release_gate,
};
use super::load::LoadedArtifacts;

pub(super) fn validate(
    artifacts: &ArtifactPaths,
    loaded: &LoadedArtifacts,
    errors: &mut Vec<String>,
) {
    let Some(artifact_root) = artifacts.source_closure.parent() else {
        errors.push("KLE release evidence has no canonical artifact root directory".to_string());
        return;
    };
    if !candidate_is_regular_file(artifacts, errors) {
        return;
    }
    let evidence_bytes = match std::fs::read(&artifacts.kle_release_evidence) {
        Ok(bytes) => bytes,
        Err(error) => {
            errors.push(format!(
                "KLE release evidence is required but missing or unreadable {}: {error}",
                artifacts.kle_release_evidence.display()
            ));
            return;
        }
    };
    let evidence = match parse_and_validate_integrity(artifact_root, &evidence_bytes) {
        Ok(evidence) => evidence,
        Err(error) => {
            errors.push(format!("KLE release evidence integrity: {error}"));
            return;
        }
    };
    let source_closure_bytes = match std::fs::read(&artifacts.source_closure) {
        Ok(bytes) => bytes,
        Err(error) => {
            errors.push(format!(
                "KLE release evidence cannot read canonical source closure {}: {error}",
                artifacts.source_closure.display()
            ));
            return;
        }
    };
    let leaf_stage_profiles = loaded
        .leaf_manifest
        .leafs
        .iter()
        .flat_map(|leaf| {
            leaf.required_profile_ids.iter().map(move |profile_id| {
                (
                    leaf.leaf_id.clone(),
                    leaf.storybook_stage_id.clone(),
                    profile_id.clone(),
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let profile_fingerprints = loaded
        .source_closure
        .profiles
        .iter()
        .map(|profile| (profile.id.clone(), profile.fingerprint.clone()))
        .collect::<BTreeMap<_, _>>();
    let canonical_root_sha256 = sha256_hex(&source_closure_bytes);
    let expected = ExpectedKleReleaseEvidence {
        canonical_root_sha256: &canonical_root_sha256,
        release_profile_matrix_fingerprint: &loaded
            .source_closure
            .root
            .release_profile_matrix_fingerprint,
        profile_fingerprints: &profile_fingerprints,
        leaf_stage_profiles: &leaf_stage_profiles,
    };
    if let Err(error) = validate_kle_release_gate(&evidence, Some(&expected)) {
        errors.push(error);
    }
}

fn candidate_is_regular_file(artifacts: &ArtifactPaths, errors: &mut Vec<String>) -> bool {
    match std::fs::symlink_metadata(&artifacts.kle_release_evidence) {
        Ok(metadata) if metadata.file_type().is_symlink() => errors.push(format!(
            "KLE release evidence must not be a symlink: {}",
            artifacts.kle_release_evidence.display()
        )),
        Ok(metadata) if !metadata.is_file() => errors.push(format!(
            "KLE release evidence must be a regular file: {}",
            artifacts.kle_release_evidence.display()
        )),
        Ok(_) => return true,
        Err(error) => errors.push(format!(
            "KLE release evidence is required but missing or unreadable {}: {error}",
            artifacts.kle_release_evidence.display()
        )),
    }
    false
}
