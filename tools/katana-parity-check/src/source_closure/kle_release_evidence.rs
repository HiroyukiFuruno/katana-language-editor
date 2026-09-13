//! KLE release-evidence candidate contract.
//!
//! The validator consumes opaque KUC writer output only. It cannot grant
//! release trust until KUC #40 exposes the public actual-show receipt producer.

use serde::{Deserialize, Serialize};

pub const CANONICAL_PROFILE_IDS: [&str; 3] = ["macos-latest", "windows-latest", "ubuntu-latest"];

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KleReleaseEvidence {
    pub schema_version: String,
    pub run_id: String,
    pub source_closure_root_sha256: String,
    pub release_profile_matrix_fingerprint: String,
    pub profiles: Vec<ReleaseProfile>,
    pub records: Vec<LeafEvidenceRecord>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReleaseProfile {
    pub profile_id: String,
    pub profile_fingerprint: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LeafEvidenceRecord {
    pub run_id: String,
    pub source_closure_root_sha256: String,
    pub leaf_id: String,
    pub profile_id: String,
    pub profile_fingerprint: String,
    pub stage_id: String,
    pub opaque_trace: BeforeAfterFiles,
    pub media: BeforeAfterFiles,
    pub outcome: OpaqueTransitOutcome,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BeforeAfterFiles {
    pub before: EvidenceFile,
    pub after: EvidenceFile,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceFile {
    pub relative_path: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum OpaqueTransitOutcome {
    SingleForwardedBatch {
        receipt: PublicKucReceipt,
    },
    NoMutation {
        before_receipt: PublicKucReceipt,
        after_receipt: PublicKucReceipt,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PublicKucReceipt {
    pub root_identity: String,
    pub presentation_revision: u64,
    pub state_revision: u64,
    pub record_hash: String,
    pub paint_plan_hash: String,
    pub accessibility_snapshot_hash: String,
    pub correlation_fingerprint: String,
    pub event_batch_fingerprint: String,
    pub event_cardinality: usize,
    pub consumed_once: bool,
}
