use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use super::kle_release_evidence::{CANONICAL_PROFILE_IDS, KleReleaseEvidence, LeafEvidenceRecord};

mod kle_release_evidence_canonical_binding;
mod kle_release_evidence_file_integrity;
mod kle_release_evidence_outcome_integrity;

pub(super) use kle_release_evidence_canonical_binding::ExpectedKleReleaseEvidence;
use kle_release_evidence_canonical_binding::validate_profiles;
use kle_release_evidence_file_integrity::validate_file_pair;
use kle_release_evidence_outcome_integrity::validate_outcome;

const SCHEMA_VERSION: &str = "1";
const SHA256_HEX_LENGTH: usize = 64;

pub(super) fn parse_and_validate_integrity(
    root: &Path,
    bytes: &[u8],
) -> Result<KleReleaseEvidence, String> {
    let evidence = serde_json::from_slice(bytes)
        .map_err(|error| format!("invalid KLE release evidence: {error}"))?;
    validate_integrity(root, &evidence)?;
    Ok(evidence)
}

pub(super) fn validate_integrity(root: &Path, evidence: &KleReleaseEvidence) -> Result<(), String> {
    validate_evidence_header(evidence)?;
    let root = root
        .canonicalize()
        .map_err(|error| format!("artifact root cannot be canonicalized: {error}"))?;
    let profiles = evidence
        .profiles
        .iter()
        .map(|profile| {
            (
                profile.profile_id.as_str(),
                profile.profile_fingerprint.as_str(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let leaf_profiles = validate_records(&root, evidence, &profiles)?;
    validate_profile_coverage(leaf_profiles)
}

pub(super) fn validate_expected_binding(
    evidence: &KleReleaseEvidence,
    expected: Option<&ExpectedKleReleaseEvidence<'_>>,
) -> Result<(), String> {
    kle_release_evidence_canonical_binding::validate_expected_binding(evidence, expected)
}

pub(super) fn validate_kle_release_gate(
    evidence: &KleReleaseEvidence,
    expected: Option<&ExpectedKleReleaseEvidence<'_>>,
) -> Result<(), String> {
    validate_expected_binding(evidence, expected)?;
    Err("KLE release evidence is blocked: KUC #40 published consumer-defined artifact plan is required".to_string())
}

fn validate_evidence_header(evidence: &KleReleaseEvidence) -> Result<(), String> {
    if evidence.schema_version != SCHEMA_VERSION {
        return Err("KLE release evidence schema must equal 1".to_string());
    }
    nonempty("run_id", &evidence.run_id)?;
    sha256(
        "source_closure_root_sha256",
        &evidence.source_closure_root_sha256,
    )?;
    nonempty(
        "release_profile_matrix_fingerprint",
        &evidence.release_profile_matrix_fingerprint,
    )?;
    validate_profiles(&evidence.profiles)?;
    if evidence.records.is_empty() {
        return Err("KLE release evidence has no leaf records".to_string());
    }
    Ok(())
}

fn validate_records<'a>(
    root: &Path,
    evidence: &'a KleReleaseEvidence,
    profiles: &BTreeMap<&str, &str>,
) -> Result<BTreeMap<&'a str, BTreeSet<&'a str>>, String> {
    let mut record_keys = BTreeSet::new();
    let mut leaf_profiles = BTreeMap::<&str, BTreeSet<&str>>::new();
    let mut stages = BTreeMap::new();
    for record in &evidence.records {
        validate_record(root, evidence, profiles, record)?;
        if !record_keys.insert((&record.leaf_id, &record.profile_id, &record.stage_id)) {
            return Err(
                "KLE release evidence has duplicate leaf/profile/stage binding".to_string(),
            );
        }
        leaf_profiles
            .entry(record.leaf_id.as_str())
            .or_default()
            .insert(record.profile_id.as_str());
        if let Some(existing_leaf) =
            stages.insert(record.stage_id.as_str(), record.leaf_id.as_str())
            && existing_leaf != record.leaf_id
        {
            return Err("KLE release evidence reuses one stage across different leafs".to_string());
        }
    }
    Ok(leaf_profiles)
}

fn validate_record(
    root: &Path,
    evidence: &KleReleaseEvidence,
    profiles: &BTreeMap<&str, &str>,
    record: &LeafEvidenceRecord,
) -> Result<(), String> {
    if record.run_id != evidence.run_id
        || record.source_closure_root_sha256 != evidence.source_closure_root_sha256
    {
        return Err("KLE release evidence record does not bind the enclosing run/root".to_string());
    }
    nonempty("leaf_id", &record.leaf_id)?;
    nonempty("stage_id", &record.stage_id)?;
    if profiles.get(record.profile_id.as_str()) != Some(&record.profile_fingerprint.as_str()) {
        return Err(
            "KLE release evidence record profile fingerprint diverges from its matrix".to_string(),
        );
    }
    validate_file_pair(root, &record.opaque_trace, "opaque trace")?;
    validate_file_pair(root, &record.media, "media")?;
    validate_outcome(&record.outcome)
}

fn validate_profile_coverage(leaf_profiles: BTreeMap<&str, BTreeSet<&str>>) -> Result<(), String> {
    let required_profiles = CANONICAL_PROFILE_IDS.into_iter().collect::<BTreeSet<_>>();
    for (leaf_id, profile_ids) in leaf_profiles {
        if profile_ids != required_profiles {
            return Err(format!(
                "KLE release evidence leaf {leaf_id} does not cover every canonical profile"
            ));
        }
    }
    Ok(())
}

fn nonempty(name: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{name} is empty"))
    } else {
        Ok(())
    }
}

fn sha256(name: &str, value: &str) -> Result<(), String> {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return Err(format!("{name} must use sha256: prefix"));
    };
    if hex.len() == SHA256_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        Ok(())
    } else {
        Err(format!("{name} is not a SHA-256 digest"))
    }
}

#[cfg(test)]
#[path = "kle_release_evidence_tests.rs"]
mod tests;
