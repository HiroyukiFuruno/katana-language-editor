use std::collections::BTreeSet;
use std::fs;

use super::{
    ExpectedKleReleaseEvidence, validate_expected_binding, validate_integrity,
    validate_kle_release_gate,
};
use crate::capability_manifest::capability_manifest_test_fixtures::{
    FixtureBuilder, FixtureDirectory,
};
use crate::source_closure::fingerprint::sha256_hex;
use crate::source_closure::kle_release_evidence::{
    BeforeAfterFiles, CANONICAL_PROFILE_IDS, EvidenceFile, KleReleaseEvidence, LeafEvidenceRecord,
    OpaqueTransitOutcome, PublicKucReceipt, ReleaseProfile,
};

#[path = "kle_release_evidence_tests/outcome.rs"]
mod outcome;

#[test]
fn rejects_schema_profile_coverage_and_unsafe_paths() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let mut candidate = evidence(&fixture)?;
    candidate.schema_version = "2".to_string();
    failure(
        validate_integrity(fixture.path(), &candidate),
        "schema must equal",
    )?;

    let mut candidate = evidence(&fixture)?;
    candidate.records.pop();
    failure(
        validate_integrity(fixture.path(), &candidate),
        "does not cover every canonical profile",
    )?;

    for path in ["../outside", "trace\\before", "C:trace/before"] {
        let mut candidate = evidence(&fixture)?;
        candidate.records[0].opaque_trace.before.relative_path = path.to_string();
        failure(
            validate_integrity(fixture.path(), &candidate),
            "safe relative path",
        )?;
    }
    Ok(())
}

#[test]
fn rejects_tampered_file_and_cross_leaf_stage_reuse() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let mut candidate = evidence(&fixture)?;
    candidate.records[0].media.before.sha256 = sha256_hex(b"not-media");
    failure(
        validate_integrity(fixture.path(), &candidate),
        "file hash does not match",
    )?;

    let mut candidate = evidence(&fixture)?;
    let additional = candidate
        .records
        .iter()
        .cloned()
        .map(|mut record| {
            record.leaf_id = "another-leaf".to_string();
            record
        })
        .collect::<Vec<_>>();
    candidate.records.extend(additional);
    failure(
        validate_integrity(fixture.path(), &candidate),
        "reuses one stage across different leafs",
    )
}

#[test]
fn rejects_missing_or_mismatched_canonical_binding_and_keeps_kuc_blocked() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let evidence = evidence(&fixture)?;
    failure(
        validate_expected_binding(&evidence, None),
        "expected canonical root/profile/leaf-stage",
    )?;

    let expected_bindings = bindings(&evidence);
    let profile_fingerprints = profile_fingerprints(&evidence);
    let wrong_root = sha256_hex(b"other-root");
    let expected = ExpectedKleReleaseEvidence {
        canonical_root_sha256: &wrong_root,
        release_profile_matrix_fingerprint: "matrix",
        profile_fingerprints: &profile_fingerprints,
        leaf_stage_profiles: &expected_bindings,
    };
    failure(
        validate_expected_binding(&evidence, Some(&expected)),
        "root differs",
    )?;

    let expected = ExpectedKleReleaseEvidence {
        canonical_root_sha256: &evidence.source_closure_root_sha256,
        release_profile_matrix_fingerprint: "other-matrix",
        profile_fingerprints: &profile_fingerprints,
        leaf_stage_profiles: &expected_bindings,
    };
    failure(
        validate_expected_binding(&evidence, Some(&expected)),
        "profile matrix differs",
    )?;

    let expected = ExpectedKleReleaseEvidence {
        canonical_root_sha256: &evidence.source_closure_root_sha256,
        release_profile_matrix_fingerprint: "matrix",
        profile_fingerprints: &profile_fingerprints,
        leaf_stage_profiles: &expected_bindings,
    };
    failure(
        validate_kle_release_gate(&evidence, Some(&expected)),
        "KUC #40",
    )
}

#[test]
fn rejects_copied_matrix_header_with_changed_profile_fingerprint() -> Result<(), String> {
    let fixture = FixtureBuilder::root()?;
    let mut evidence = evidence(&fixture)?;
    let expected_bindings = bindings(&evidence);
    let profile_fingerprints = profile_fingerprints(&evidence);
    evidence.profiles[0].profile_fingerprint = "invented-fingerprint".to_string();
    for record in evidence
        .records
        .iter_mut()
        .filter(|record| record.profile_id == CANONICAL_PROFILE_IDS[0])
    {
        record.profile_fingerprint = "invented-fingerprint".to_string();
    }
    let expected = ExpectedKleReleaseEvidence {
        canonical_root_sha256: &evidence.source_closure_root_sha256,
        release_profile_matrix_fingerprint: "matrix",
        profile_fingerprints: &profile_fingerprints,
        leaf_stage_profiles: &expected_bindings,
    };
    failure(
        validate_expected_binding(&evidence, Some(&expected)),
        "profile fingerprints differ",
    )
}

#[cfg(unix)]
#[test]
fn rejects_symlinked_parent_directory() -> Result<(), String> {
    use std::os::unix::fs::symlink;

    let fixture = FixtureBuilder::root()?;
    let evidence = evidence(&fixture)?;
    fs::create_dir(fixture.path().join("outside")).map_err(|error| error.to_string())?;
    fs::write(fixture.path().join("outside/before"), b"before")
        .map_err(|error| error.to_string())?;
    symlink(
        fixture.path().join("outside"),
        fixture.path().join("trace-link"),
    )
    .map_err(|error| error.to_string())?;
    let mut evidence = evidence;
    evidence.records[0].opaque_trace.before.relative_path = "trace-link/before".to_string();
    failure(
        validate_integrity(fixture.path(), &evidence),
        "path contains a symlink",
    )
}

fn evidence(fixture: &FixtureDirectory) -> Result<KleReleaseEvidence, String> {
    for directory in ["trace", "media"] {
        let path = fixture.path().join(directory);
        if !path.is_dir() {
            fs::create_dir(&path).map_err(|error| error.to_string())?;
        }
    }
    for (path, bytes) in [
        ("trace/before", b"before".as_slice()),
        ("trace/after", b"after".as_slice()),
        ("media/before", b"before".as_slice()),
        ("media/after", b"after".as_slice()),
    ] {
        fs::write(fixture.path().join(path), bytes).map_err(|error| error.to_string())?;
    }
    let profiles = CANONICAL_PROFILE_IDS
        .into_iter()
        .map(|profile_id| ReleaseProfile {
            profile_id: profile_id.to_string(),
            profile_fingerprint: format!("fingerprint-{profile_id}"),
        })
        .collect::<Vec<_>>();
    let root = sha256_hex(b"root");
    let records = profiles
        .iter()
        .map(|profile| LeafEvidenceRecord {
            run_id: "run".to_string(),
            source_closure_root_sha256: root.clone(),
            leaf_id: "leaf".to_string(),
            profile_id: profile.profile_id.clone(),
            profile_fingerprint: profile.profile_fingerprint.clone(),
            stage_id: "leaf-stage".to_string(),
            opaque_trace: pair("trace/before", "trace/after"),
            media: pair("media/before", "media/after"),
            outcome: OpaqueTransitOutcome::SingleForwardedBatch { receipt: receipt() },
        })
        .collect();
    Ok(KleReleaseEvidence {
        schema_version: "1".to_string(),
        run_id: "run".to_string(),
        source_closure_root_sha256: root,
        release_profile_matrix_fingerprint: "matrix".to_string(),
        profiles,
        records,
    })
}

fn bindings(evidence: &KleReleaseEvidence) -> BTreeSet<(String, String, String)> {
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

fn profile_fingerprints(
    evidence: &KleReleaseEvidence,
) -> std::collections::BTreeMap<String, String> {
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

fn pair(before: &str, after: &str) -> BeforeAfterFiles {
    BeforeAfterFiles {
        before: EvidenceFile {
            relative_path: before.to_string(),
            sha256: sha256_hex(before.rsplit('/').next().unwrap_or_default().as_bytes()),
        },
        after: EvidenceFile {
            relative_path: after.to_string(),
            sha256: sha256_hex(after.rsplit('/').next().unwrap_or_default().as_bytes()),
        },
    }
}

fn receipt() -> PublicKucReceipt {
    PublicKucReceipt {
        root_identity: "root".to_string(),
        presentation_revision: 1,
        state_revision: 1,
        record_hash: "record".to_string(),
        paint_plan_hash: "paint".to_string(),
        accessibility_snapshot_hash: "accesskit".to_string(),
        correlation_fingerprint: "correlation".to_string(),
        event_batch_fingerprint: "events".to_string(),
        event_cardinality: 1,
        consumed_once: true,
    }
}

fn empty_receipt() -> PublicKucReceipt {
    PublicKucReceipt {
        event_cardinality: 0,
        ..receipt()
    }
}

fn failure(result: Result<(), String>, needle: &str) -> Result<(), String> {
    match result {
        Err(value) if value.contains(needle) => Ok(()),
        Err(value) => Err(format!("expected {needle}, got {value}")),
        Ok(()) => Err(format!("expected {needle}")),
    }
}
