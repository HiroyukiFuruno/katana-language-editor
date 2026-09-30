use super::super::artifact_model::BranchRecord;
use super::super::fingerprint::sha256_hex;
use super::super::root_model::ManifestRoot;
use super::super::source_requirement_alias_ledger::AliasEntry;
use super::types::{
    FingerprintInput, RequirementBinding, RequirementBindingResult, UnboundBranch,
    UnboundInactiveProfilePredicate,
};

impl RequirementBindingResult {
    pub(super) fn fingerprint_value(&self) -> Result<String, serde_json::Error> {
        let digest = FingerprintInput {
            root: &self.root,
            bindings: &self.bindings,
            unresolved: &self.unresolved,
            unbound_branches: &self.unbound_branches,
        };
        serde_json::to_vec(&digest).map(|bytes| sha256_hex(&bytes))
    }
}

pub(super) fn binding(
    entry: &AliasEntry,
    source_sha256: &str,
    branch: &BranchRecord,
) -> RequirementBinding {
    RequirementBinding {
        requirement_id: entry.requirement_id.clone(),
        short_reference: entry.short_reference.clone(),
        source_path: entry.source_path.clone(),
        source_sha256: source_sha256.to_string(),
        branch_id: branch.branch_id.clone(),
        span_start_line: branch.span.start_line,
        span_end_line: branch.span.end_line,
        source_excerpt_sha256: branch.source_excerpt_sha256.clone(),
    }
}

pub(super) fn unbound_branch(branch: &BranchRecord) -> UnboundBranch {
    UnboundBranch {
        branch_id: branch.branch_id.clone(),
        file: branch.file.clone(),
        symbol: branch.symbol.clone(),
        span_start_line: branch.span.start_line,
        span_end_line: branch.span.end_line,
        kind: branch.kind.clone(),
        condition: branch.condition.clone(),
        active_profile_ids: branch.active_profile_ids.clone(),
        inactive_profile_predicates: branch
            .inactive_profile_predicates
            .iter()
            .map(|predicate| UnboundInactiveProfilePredicate {
                profile_id: predicate.profile_id.clone(),
                predicate: predicate.predicate.clone(),
            })
            .collect(),
        incoming_edges: branch.incoming_edges.clone(),
        source_excerpt_sha256: branch.source_excerpt_sha256.clone(),
    }
}

pub(super) fn ensure_same_root(
    source: &ManifestRoot,
    branches: &ManifestRoot,
) -> Result<(), String> {
    if source.schema_version != branches.schema_version
        || source.katana_revision != branches.katana_revision
        || source.katana_tree_fingerprint != branches.katana_tree_fingerprint
        || source.katana_external_ui_fingerprint != branches.katana_external_ui_fingerprint
        || source.user_mandated_extensions_fingerprint
            != branches.user_mandated_extensions_fingerprint
        || source.source_universe_fingerprint != branches.source_universe_fingerprint
        || source.requirement_source_aliases_fingerprint
            != branches.requirement_source_aliases_fingerprint
        || source.kle_tree_fingerprint != branches.kle_tree_fingerprint
        || source.kuc_tree_fingerprint != branches.kuc_tree_fingerprint
        || source.release_profile_matrix_fingerprint != branches.release_profile_matrix_fingerprint
        || source.generator_fingerprint != branches.generator_fingerprint
        || source.generated_at_utc != branches.generated_at_utc
        || source.static_leaf_count != branches.static_leaf_count
        || source.expected_leaf_ids != branches.expected_leaf_ids
    {
        return Err("source-closure roots differ for requirement binding".to_string());
    }
    Ok(())
}
