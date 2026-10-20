use super::model::{Bundle, ProfileEvidence};
use super::profiles::{index_profiles, validate_profile_fingerprints};

fn profile(profile_id: &str) -> ProfileEvidence {
    ProfileEvidence {
        profile_id: profile_id.into(),
        profile_fingerprint: format!("sha256:{profile_id}"),
        plan_sha256: "sha256:plan".into(),
        evidence_sha256: "sha256:evidence".into(),
    }
}

#[test]
fn bundle_rejects_unknown_fields() {
    let result = serde_json::from_str::<Bundle>(
        r#"{"schema":"kle.consumer-release-evidence.v1","run_id":"run","source_closure_sha256":"sha256:x","mapping_sha256":"sha256:y","profiles":[],"extra":true}"#,
    );
    assert!(result.is_err());
}

#[test]
fn profile_set_rejects_missing_duplicate_and_unknown_profiles() {
    assert!(index_profiles(&[profile("macos-latest"), profile("windows-latest")]).is_err());
    assert!(
        index_profiles(&[
            profile("macos-latest"),
            profile("macos-latest"),
            profile("ubuntu-latest")
        ])
        .is_err()
    );
    assert!(
        index_profiles(&[
            profile("macos-latest"),
            profile("windows-latest"),
            profile("unknown")
        ])
        .is_err()
    );
}

#[test]
fn profile_fingerprint_requires_one_matching_source_record() {
    let profiles = [
        profile("macos-latest"),
        profile("windows-latest"),
        profile("ubuntu-latest"),
    ];
    let result =
        index_profiles(&profiles).and_then(|indexed| validate_profile_fingerprints(&[], &indexed));
    assert!(result.is_err());
}

#[test]
fn reconstructed_arrays_reject_non_canonical_order() {
    assert!(super::compare_values("profiles", &["macos", "ubuntu"], &["ubuntu", "macos"]).is_err());
    assert!(super::compare_values("profiles", &["macos", "ubuntu"], &["macos"]).is_err());
    assert!(
        super::compare_values("profiles", &["macos", "ubuntu"], &["macos", "tampered"]).is_err()
    );
}
