use serde::Serialize;

use super::fingerprint::sha256_hex;
use super::operational_input::{
    EvidenceRef, KatanaTreeEvidence, ProfileProbeInput, RootProvenance, SourceClosureInput,
};

fn canonical_identity_bytes<T: Serialize>(value: &T, context: &str) -> Vec<u8> {
    match serde_json::to_vec(value) {
        Ok(bytes) => bytes,
        Err(error) => format!("serialization-error:{context}:{error}").into_bytes(),
    }
}

#[derive(Serialize)]
struct EvidenceIdentity<'a> {
    capture_id: &'a str,
    path: &'a str,
    sha256: &'a str,
    command_or_source: &'a str,
    runner_label: &'a str,
    exit_status: i32,
}

#[derive(Serialize)]
struct ProfileIdentity<'a> {
    id: &'a str,
    runner_label: &'a str,
    katana_revision: &'a str,
    rustc_host_triple: &'a str,
    rustc_vv_raw: EvidenceIdentity<'a>,
    rustc_cfg_raw: EvidenceIdentity<'a>,
    cargo_resolution_raw: EvidenceIdentity<'a>,
    cargo_lock_raw: EvidenceIdentity<'a>,
    source_tree_fingerprint: &'a str,
    source_tree: Vec<EvidenceIdentity<'a>>,
    active_edge_ids: &'a [String],
    inactive_cfg_edges: &'a [super::operational_input::InactiveCfgEdge],
    cfg_edge_probe: EvidenceIdentity<'a>,
}

#[derive(Serialize)]
struct RootIdentity<'a> {
    schema_version: &'a str,
    katana_revision: &'a str,
    kle_revision: &'a str,
    kle_worktree_clean: bool,
    kuc_revision: &'a str,
    kuc_worktree_clean: bool,
    katana_tree_fingerprint: &'a str,
    katana_external_ui_fingerprint: &'a str,
    user_mandated_extensions_fingerprint: &'a str,
    source_universe_fingerprint: &'a str,
    requirement_source_aliases_fingerprint: &'a str,
    kle_tree_fingerprint: &'a str,
    kuc_tree_fingerprint: &'a str,
    release_profile_matrix_fingerprint: &'a str,
    generator_fingerprint: &'a str,
    evidence: &'a super::operational_input::RootEvidence,
}

fn evidence_identity(evidence: &EvidenceRef) -> EvidenceIdentity<'_> {
    EvidenceIdentity {
        capture_id: &evidence.capture_id,
        path: &evidence.path,
        sha256: &evidence.sha256,
        command_or_source: &evidence.command_or_source,
        runner_label: &evidence.runner_label,
        exit_status: evidence.exit_status,
    }
}

impl ProfileProbeInput {
    pub(crate) fn identity_fingerprint(&self) -> String {
        let identity = ProfileIdentity {
            id: &self.id,
            runner_label: &self.runner_label,
            katana_revision: &self.katana_revision,
            rustc_host_triple: &self.rustc_host_triple,
            rustc_vv_raw: evidence_identity(&self.rustc_vv_raw),
            rustc_cfg_raw: evidence_identity(&self.rustc_cfg_raw),
            cargo_resolution_raw: evidence_identity(&self.cargo_resolution_raw),
            cargo_lock_raw: evidence_identity(&self.cargo_lock_raw),
            source_tree_fingerprint: &self.source_tree_fingerprint,
            source_tree: self.source_tree.iter().map(evidence_identity).collect(),
            active_edge_ids: &self.active_edge_ids,
            inactive_cfg_edges: &self.inactive_cfg_edges,
            cfg_edge_probe: evidence_identity(&self.cfg_edge_probe),
        };
        sha256_hex(&canonical_identity_bytes(&identity, "profile"))
    }
}

impl RootProvenance {
    pub(crate) fn identity_fingerprint(&self) -> String {
        let identity = RootIdentity {
            schema_version: &self.schema_version,
            katana_revision: &self.katana_revision,
            kle_revision: &self.kle_revision,
            kle_worktree_clean: self.kle_worktree_clean,
            kuc_revision: &self.kuc_revision,
            kuc_worktree_clean: self.kuc_worktree_clean,
            katana_tree_fingerprint: &self.katana_tree_fingerprint,
            katana_external_ui_fingerprint: &self.katana_external_ui_fingerprint,
            user_mandated_extensions_fingerprint: &self.user_mandated_extensions_fingerprint,
            source_universe_fingerprint: &self.source_universe_fingerprint,
            requirement_source_aliases_fingerprint: &self.requirement_source_aliases_fingerprint,
            kle_tree_fingerprint: &self.kle_tree_fingerprint,
            kuc_tree_fingerprint: &self.kuc_tree_fingerprint,
            release_profile_matrix_fingerprint: &self.release_profile_matrix_fingerprint,
            generator_fingerprint: &self.generator_fingerprint,
            evidence: &self.evidence,
        };
        sha256_hex(&canonical_identity_bytes(&identity, "root"))
    }
}

impl SourceClosureInput {
    pub(crate) fn matrix_fingerprint(&self) -> String {
        let profiles = self
            .profile_probes
            .iter()
            .map(ProfileProbeInput::identity_fingerprint)
            .collect::<Vec<_>>();
        sha256_hex(&canonical_identity_bytes(&profiles, "profile-matrix"))
    }

    pub(crate) fn identity_fingerprint(&self) -> String {
        #[derive(Serialize)]
        struct InputIdentity<'a> {
            input_schema_version: &'a str,
            katana_seed_paths: &'a [String],
            root_identity: String,
            profile_identities: Vec<String>,
        }
        let identity = InputIdentity {
            input_schema_version: &self.input_schema_version,
            katana_seed_paths: &self.katana_seed_paths,
            root_identity: self.root.identity_fingerprint(),
            profile_identities: self
                .profile_probes
                .iter()
                .map(ProfileProbeInput::identity_fingerprint)
                .collect(),
        };
        sha256_hex(&canonical_identity_bytes(&identity, "input"))
    }
}

impl EvidenceRef {
    pub(crate) fn set_fingerprint(evidence: &[Self]) -> String {
        let mut records = evidence
            .iter()
            .map(|entry| format!("{}\0{}\n", entry.path, entry.sha256))
            .collect::<Vec<_>>();
        records.sort_unstable();
        sha256_hex(records.concat().as_bytes())
    }

    pub(crate) fn profile_tree_fingerprint(profile_id: &str, evidence: &[Self]) -> String {
        let prefix = format!("profiles/{profile_id}/raw/source-tree/");
        let mut records = evidence
            .iter()
            .map(|entry| {
                let source_path = match entry
                    .path
                    .strip_prefix(&prefix)
                    .or_else(|| entry.path.strip_prefix(&format!("evidence/{prefix}")))
                {
                    Some(path) => path,
                    None => entry.path.as_str(),
                };
                format!("{source_path}\0{}\n", entry.sha256)
            })
            .collect::<Vec<_>>();
        records.sort_unstable();
        sha256_hex(records.concat().as_bytes())
    }
}

impl KatanaTreeEvidence {
    pub(crate) fn set_fingerprint(evidence: &[Self]) -> String {
        let mut records = evidence
            .iter()
            .map(|entry| format!("{}\0{}\n", entry.source_path, entry.evidence.sha256))
            .collect::<Vec<_>>();
        records.sort_unstable();
        sha256_hex(records.concat().as_bytes())
    }
}
