use super::super::kle_release_evidence::{
    CANONICAL_PROFILE_IDS, KleReleaseEvidence, ReleaseProfile,
};
use std::collections::{BTreeMap, BTreeSet};

pub(in crate::source_closure) struct ExpectedKleReleaseEvidence<'a> {
    pub(in crate::source_closure) canonical_root_sha256: &'a str,
    pub(in crate::source_closure) release_profile_matrix_fingerprint: &'a str,
    pub(in crate::source_closure) profile_fingerprints: &'a BTreeMap<String, String>,
    pub(in crate::source_closure) leaf_stage_profiles: &'a BTreeSet<(String, String, String)>,
}

pub(super) fn validate_profiles(profiles: &[ReleaseProfile]) -> Result<(), String> {
    if profiles.len() != CANONICAL_PROFILE_IDS.len() {
        return Err("KLE release evidence requires the complete three-profile matrix".to_string());
    }
    for (profile, expected_id) in profiles.iter().zip(CANONICAL_PROFILE_IDS) {
        if profile.profile_id != expected_id {
            return Err("KLE release evidence profile matrix is not canonical".to_string());
        }
        super::nonempty("profile_fingerprint", &profile.profile_fingerprint)?;
    }
    Ok(())
}

pub(super) fn validate_expected_binding(
    evidence: &KleReleaseEvidence,
    expected: Option<&ExpectedKleReleaseEvidence<'_>>,
) -> Result<(), String> {
    let expected = expected.ok_or_else(|| {
        "KLE release evidence expected canonical root/profile/leaf-stage bindings are required"
            .to_string()
    })?;
    if evidence.source_closure_root_sha256 != expected.canonical_root_sha256 {
        return Err(
            "KLE release evidence root differs from the canonical source closure".to_string(),
        );
    }
    if evidence.release_profile_matrix_fingerprint != expected.release_profile_matrix_fingerprint {
        return Err(
            "KLE release evidence profile matrix differs from the canonical source closure"
                .to_string(),
        );
    }
    if actual_profile_fingerprints(evidence) != *expected.profile_fingerprints {
        return Err(
            "KLE release evidence profile fingerprints differ from the canonical source closure"
                .to_string(),
        );
    }
    if actual_leaf_stage_profiles(evidence) != *expected.leaf_stage_profiles {
        return Err("KLE release evidence leaf/stage/profile bindings differ from the canonical source closure".to_string());
    }
    Ok(())
}

fn actual_profile_fingerprints(evidence: &KleReleaseEvidence) -> BTreeMap<String, String> {
    evidence
        .profiles
        .iter()
        .map(|profile| {
            (
                profile.profile_id.clone(),
                profile.profile_fingerprint.clone(),
            )
        })
        .collect()
}
fn actual_leaf_stage_profiles(evidence: &KleReleaseEvidence) -> BTreeSet<(String, String, String)> {
    evidence
        .records
        .iter()
        .map(|record| {
            (
                record.leaf_id.clone(),
                record.stage_id.clone(),
                record.profile_id.clone(),
            )
        })
        .collect()
}
