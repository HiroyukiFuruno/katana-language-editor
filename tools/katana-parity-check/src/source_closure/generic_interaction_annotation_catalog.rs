use std::collections::BTreeMap;

use serde::Serialize;

use super::artifact_model::BranchCatalogArtifact;
use super::canonical_leaf_binding::CanonicalLeafBindingEvidence;

pub(crate) const SCHEMA: &str = "generic-interaction-annotation-catalog.v1";

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct GenericInteractionAnnotationCatalog {
    schema: &'static str,
    entries: Vec<GenericInteractionAnnotationEntry>,
}

#[derive(Debug, Serialize)]
#[serde(deny_unknown_fields)]
struct GenericInteractionAnnotationEntry {
    leaf_id: String,
    source_candidate_fingerprint: String,
    requirement_origin_span: String,
    source_branch_id: String,
    action_origin_id: String,
    required_profile_ids: Vec<String>,
    source_path: String,
    source_sha256: String,
    branch_span_start_line: usize,
    branch_span_end_line: usize,
    branch_excerpt_sha256: String,
}

impl GenericInteractionAnnotationCatalog {
    pub(crate) fn build(
        evidence: &CanonicalLeafBindingEvidence,
        branches: &BranchCatalogArtifact,
    ) -> Result<Self, String> {
        let projected = evidence.project_leafs(branches)?;
        let mut candidates = BTreeMap::new();
        for candidate in &evidence.structured.candidates {
            if candidates
                .insert(candidate.identity_fingerprint.as_str(), candidate)
                .is_some()
            {
                return Err("structured candidates have duplicate identities".into());
            }
        }
        let mut entries = projected
            .into_iter()
            .map(|leaf| {
                let candidate = candidates
                    .get(leaf.source_candidate_fingerprint.as_str())
                    .ok_or_else(|| "projected leaf has no structured candidate".to_string())?;
                let branch_id = singleton(&leaf.source_branches, "source branch")?;
                let action_origin_id = singleton(&leaf.action_origin_ids, "action origin")?;
                let binding = &candidate.requirement_binding;
                Ok(GenericInteractionAnnotationEntry {
                    leaf_id: leaf.leaf_id,
                    source_candidate_fingerprint: leaf.source_candidate_fingerprint,
                    requirement_origin_span: leaf.requirement_origin.span,
                    source_branch_id: branch_id.to_owned(),
                    action_origin_id: action_origin_id.to_owned(),
                    required_profile_ids: leaf.required_profile_ids,
                    source_path: binding.source_path.clone(),
                    source_sha256: binding.source_sha256.clone(),
                    branch_span_start_line: binding.span_start_line,
                    branch_span_end_line: binding.span_end_line,
                    branch_excerpt_sha256: binding.source_excerpt_sha256.clone(),
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        entries.sort_by(|left, right| left.leaf_id.cmp(&right.leaf_id));
        Ok(Self {
            schema: SCHEMA,
            entries,
        })
    }
}

fn singleton<'a>(values: &'a [String], name: &str) -> Result<&'a str, String> {
    match values {
        [value] if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("source candidate does not have exactly one {name}")),
    }
}

#[cfg(test)]
#[path = "generic_interaction_annotation_catalog_tests.rs"]
mod tests;
