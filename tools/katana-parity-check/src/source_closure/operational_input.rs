use serde::{Deserialize, Serialize};

use super::root_model::ManifestRoot;

pub(crate) const INPUT_SCHEMA_VERSION: &str = "1";
pub(crate) const FIXED_KATANA_REVISION: &str = "4f6a6287c650a38633c7baeb544a92e739c68567";
pub(crate) const FULL_GIT_SHA_LENGTH: usize = 40;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceClosureInput {
    pub(crate) input_schema_version: String,
    pub(crate) katana_seed_paths: Vec<String>,
    pub(crate) root: RootProvenance,
    pub(crate) profile_probes: Vec<ProfileProbeInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EvidenceRef {
    pub(crate) capture_id: String,
    pub(crate) path: String,
    pub(crate) sha256: String,
    pub(crate) command_or_source: String,
    pub(crate) runner_label: String,
    pub(crate) exit_status: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct KatanaTreeEvidence {
    pub(crate) source_path: String,
    #[serde(flatten)]
    pub(crate) evidence: EvidenceRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RootProvenance {
    pub(crate) schema_version: String,
    pub(crate) katana_revision: String,
    pub(crate) kle_revision: String,
    pub(crate) kle_worktree_clean: bool,
    pub(crate) kuc_revision: String,
    pub(crate) kuc_worktree_clean: bool,
    pub(crate) katana_tree_fingerprint: String,
    pub(crate) katana_external_ui_fingerprint: String,
    pub(crate) user_mandated_extensions_fingerprint: String,
    pub(crate) source_universe_fingerprint: String,
    pub(crate) requirement_source_aliases_fingerprint: String,
    pub(crate) kle_tree_fingerprint: String,
    pub(crate) kuc_tree_fingerprint: String,
    pub(crate) release_profile_matrix_fingerprint: String,
    pub(crate) generator_fingerprint: String,
    pub(crate) generated_at_utc: String,
    pub(crate) evidence: RootEvidence,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RootEvidence {
    pub(crate) katana_revision: EvidenceRef,
    pub(crate) kle_revision: EvidenceRef,
    pub(crate) kle_worktree_status: EvidenceRef,
    pub(crate) kuc_revision: EvidenceRef,
    pub(crate) kuc_worktree_status: EvidenceRef,
    pub(crate) katana_tree: Vec<KatanaTreeEvidence>,
    pub(crate) katana_external_ui: Vec<EvidenceRef>,
    pub(crate) user_mandated_extensions: EvidenceRef,
    pub(crate) source_universe: EvidenceRef,
    pub(crate) requirement_source_aliases: EvidenceRef,
    pub(crate) kle_tree: Vec<EvidenceRef>,
    pub(crate) kuc_tree: Vec<EvidenceRef>,
    pub(crate) release_profile_matrix: Vec<EvidenceRef>,
    pub(crate) generator_binary: EvidenceRef,
    pub(crate) generator_schema: EvidenceRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileProbeInput {
    pub(crate) id: String,
    pub(crate) runner_label: String,
    pub(crate) katana_revision: String,
    pub(crate) fingerprint: String,
    pub(crate) rustc_host_triple: String,
    pub(crate) rustc_vv_raw: EvidenceRef,
    pub(crate) rustc_cfg_raw: EvidenceRef,
    pub(crate) cargo_resolution_raw: EvidenceRef,
    pub(crate) cargo_lock_raw: EvidenceRef,
    pub(crate) source_tree_fingerprint: String,
    pub(crate) source_tree: Vec<EvidenceRef>,
    pub(crate) active_edge_ids: Vec<String>,
    pub(crate) inactive_cfg_edges: Vec<InactiveCfgEdge>,
    pub(crate) cfg_edge_probe: EvidenceRef,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct InactiveCfgEdge {
    pub(crate) edge_id: String,
    pub(crate) predicate: String,
    pub(crate) span: String,
}

impl RootProvenance {
    pub(crate) fn to_manifest_root(&self) -> ManifestRoot {
        ManifestRoot {
            schema_version: self.schema_version.clone(),
            katana_revision: self.katana_revision.clone(),
            katana_tree_fingerprint: self.katana_tree_fingerprint.clone(),
            katana_external_ui_fingerprint: self.katana_external_ui_fingerprint.clone(),
            user_mandated_extensions_fingerprint: self.user_mandated_extensions_fingerprint.clone(),
            source_universe_fingerprint: self.source_universe_fingerprint.clone(),
            requirement_source_aliases_fingerprint: self
                .requirement_source_aliases_fingerprint
                .clone(),
            kle_tree_fingerprint: self.kle_tree_fingerprint.clone(),
            kuc_tree_fingerprint: self.kuc_tree_fingerprint.clone(),
            release_profile_matrix_fingerprint: self.release_profile_matrix_fingerprint.clone(),
            generator_fingerprint: self.generator_fingerprint.clone(),
            generated_at_utc: self.generated_at_utc.clone(),
            static_leaf_count: None,
            expected_leaf_ids: None,
        }
    }
}

impl SourceClosureInput {
    pub(crate) fn canonical_json_bytes(&self) -> Result<Vec<u8>, String> {
        let mut bytes = serde_json::to_vec_pretty(self).map_err(|error| {
            format!("failed to serialize canonical SourceClosureInput: {error}")
        })?;
        bytes.push(b'\n');
        Ok(bytes)
    }
}
