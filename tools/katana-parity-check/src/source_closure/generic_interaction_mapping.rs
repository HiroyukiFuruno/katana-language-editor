use std::path::Path;

use serde::Deserialize;

use super::artifact_model::{BranchCatalogArtifact, LeafStatusRecord};
use super::canonical_leaf_binding::CanonicalLeafBindingEvidence;

mod validation;

#[cfg(test)]
use validation::SCHEMA;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenericInteractionMapping {
    schema: String,
    mappings: Vec<GenericInteractionMappingEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GenericInteractionMappingEntry {
    source_candidate_fingerprint: String,
    requirement_origin_span: String,
    source_branch_id: String,
    action_origin_id: String,
    required_profile_ids: Vec<String>,
    interaction: String,
    visible_path: Vec<String>,
    state_preconditions: Vec<String>,
    kuc_component: String,
    kle_public_show_signature: String,
    kle_opaque_transit: String,
    declared_effect_kind: String,
    semantic_evidence: Vec<SemanticEvidence>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SemanticEvidence {
    source_path: String,
    source_sha256: String,
    branch_id: String,
    span_start_line: usize,
    span_end_line: usize,
    source_excerpt_sha256: String,
}

impl GenericInteractionMapping {
    pub(crate) fn enrich(
        &self,
        evidence: &CanonicalLeafBindingEvidence,
        branches: &BranchCatalogArtifact,
    ) -> Result<Vec<LeafStatusRecord>, String> {
        validation::validate_schema(self)?;
        let mut projected = evidence.project_leafs(branches)?;
        let entries = validation::index_entries(self)?;
        let candidates = validation::candidate_index(evidence)?;
        if entries.len() != projected.len() {
            return Err("generic interaction mapping is not total over source candidates".into());
        }
        for leaf in &mut projected {
            let entry = entries
                .get(leaf.source_candidate_fingerprint.as_str())
                .ok_or_else(|| {
                    format!(
                        "generic interaction mapping is missing source candidate {}",
                        leaf.source_candidate_fingerprint
                    )
                })?;
            let candidate = candidates
                .get(leaf.source_candidate_fingerprint.as_str())
                .ok_or_else(|| "projected leaf has no structured candidate".to_string())?;
            validation::validate_identity(leaf, entry, candidate)?;
            validation::validate_semantics(entry)?;
            leaf.visible_path = entry.visible_path.clone();
            leaf.state_preconditions = entry.state_preconditions.clone();
            leaf.kuc_component = entry.kuc_component.clone();
            leaf.kle_public_show_signature = entry.kle_public_show_signature.clone();
            leaf.kle_opaque_transit = entry.kle_opaque_transit.clone();
            leaf.declared_effect_kind = entry.declared_effect_kind.clone();
            leaf.storybook_stage_id = format!("kuc-stage:{}", leaf.source_candidate_fingerprint);
            leaf.status = "mapped_pending_evidence".into();
        }
        Ok(projected)
    }

    pub(crate) fn read(path: &Path) -> Result<Self, String> {
        let bytes = std::fs::read(path)
            .map_err(|error| format!("failed to read generic interaction mapping: {error}"))?;
        serde_json::from_slice(&bytes)
            .map_err(|error| format!("failed to parse generic interaction mapping: {error}"))
    }
}

#[cfg(test)]
#[path = "generic_interaction_mapping_tests.rs"]
mod tests;
