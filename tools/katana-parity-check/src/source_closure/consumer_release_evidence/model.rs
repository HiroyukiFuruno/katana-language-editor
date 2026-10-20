use serde::Deserialize;

pub(super) const SCHEMA: &str = "kle.consumer-release-evidence.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Bundle {
    pub(super) schema: String,
    pub(super) run_id: String,
    pub(super) source_closure_sha256: String,
    pub(super) mapping_sha256: String,
    pub(super) profiles: Vec<ProfileEvidence>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ProfileEvidence {
    pub(super) profile_id: String,
    pub(super) profile_fingerprint: String,
    pub(super) plan_sha256: String,
    pub(super) evidence_sha256: String,
}
