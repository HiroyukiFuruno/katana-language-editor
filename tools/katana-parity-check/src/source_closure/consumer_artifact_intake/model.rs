use serde::Deserialize;

pub(super) const PLAN_SCHEMA: &str = "kle.consumer-artifact-plan.v1";
pub(super) const EVIDENCE_SCHEMA: &str = "kle-consumer-artifact-evidence.v1";
pub(super) const EVIDENCE_FILE: &str = "kle-consumer-artifact-evidence.json";
pub(super) const KUC_SCHEMA: &str = "kuc.consumer-full-editor-artifact.v1";
pub(super) const KUC_SCHEMA_VERSION: u16 = 1;
pub(super) const KUC_STAGE_PREFIX: &str = "consumer-stage-";
pub(super) const INITIAL_ROOT_REVISION: u64 = 1;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Plan {
    pub(super) schema: String,
    pub(super) run_id: String,
    pub(super) source_closure_sha256: String,
    pub(super) profile_fingerprint: String,
    pub(super) profile_id: String,
    pub(super) stages: Vec<PlanStage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct PlanStage {
    pub(super) leaf_id: String,
    pub(super) interaction: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Evidence {
    pub(super) schema: String,
    pub(super) run_id: String,
    pub(super) source_closure_sha256: String,
    pub(super) profile_fingerprint: String,
    pub(super) profile_id: String,
    pub(super) stages: Vec<EvidenceStage>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct EvidenceStage {
    pub(super) leaf_id: String,
    pub(super) interaction: String,
    pub(super) stage_id: String,
    pub(super) media_path: String,
    pub(super) media_sha256: String,
    pub(super) pixel_hash: String,
    pub(super) root_record_hash: String,
    pub(super) accesskit_snapshot_hash: String,
    pub(super) unicode_evidence_hash: String,
    pub(super) root_revision: u64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct KucManifest {
    pub(super) schema: String,
    pub(super) schema_version: u16,
    pub(super) stage_id: String,
    pub(super) leaf_id: String,
    pub(super) root_revision: u64,
    pub(super) png_sha256: String,
    pub(super) pixel_hash: String,
    pub(super) root_record_hash: String,
    pub(super) accesskit_snapshot_hash: String,
    pub(super) unicode_evidence_hash: String,
    pub(super) unicode_evidence_payload_sha256: String,
    pub(super) unicode_evidence_file: String,
    pub(super) forwarding_receipt_fingerprint: String,
}
