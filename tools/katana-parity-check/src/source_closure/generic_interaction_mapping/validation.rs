use std::collections::BTreeMap;

use super::super::artifact_model::LeafStatusRecord;
use super::super::canonical_leaf_binding::CanonicalLeafBindingEvidence;
use super::super::structured_leaf_candidates::StructuredLeafCandidate;
use super::super::validator_helpers::{is_canonical_profile_set_list, is_placeholder};
use super::{GenericInteractionMapping, GenericInteractionMappingEntry};

pub(crate) const SCHEMA: &str = "generic-interaction-mapping-v2";
const KUC_INTERACTIONS: [&str; 10] = [
    "text_input",
    "ime_commit",
    "selection",
    "scroll",
    "toolbar_activation",
    "floating_toolbar",
    "search",
    "context_menu",
    "accessibility_activation",
    "viewport_resize",
];
const EFFECT_KINDS: [&str; 3] = [
    "kuc_retained_ui_effect",
    "in_process_host_effect",
    "native_external_host_effect",
];

pub(super) fn validate_schema(mapping: &GenericInteractionMapping) -> Result<(), String> {
    if mapping.schema == SCHEMA {
        Ok(())
    } else {
        Err(format!(
            "unsupported generic interaction mapping schema: {}",
            mapping.schema
        ))
    }
}

pub(super) fn index_entries(
    mapping: &GenericInteractionMapping,
) -> Result<BTreeMap<&str, &GenericInteractionMappingEntry>, String> {
    let mut entries = BTreeMap::new();
    for entry in &mapping.mappings {
        if entry.source_candidate_fingerprint.trim().is_empty()
            || entries
                .insert(entry.source_candidate_fingerprint.as_str(), entry)
                .is_some()
        {
            return Err(
                "generic interaction mapping has duplicate or empty candidate identity".into(),
            );
        }
    }
    Ok(entries)
}

pub(super) fn candidate_index(
    evidence: &CanonicalLeafBindingEvidence,
) -> Result<BTreeMap<&str, &StructuredLeafCandidate>, String> {
    let mut candidates = BTreeMap::new();
    for candidate in &evidence.structured.candidates {
        if candidates
            .insert(candidate.identity_fingerprint.as_str(), candidate)
            .is_some()
        {
            return Err("structured candidates have duplicate identities".into());
        }
    }
    Ok(candidates)
}

pub(super) fn validate_identity(
    leaf: &LeafStatusRecord,
    entry: &GenericInteractionMappingEntry,
    candidate: &StructuredLeafCandidate,
) -> Result<(), String> {
    let branch_ids = singleton(&leaf.source_branches, "source branch")?;
    let action_ids = singleton(&leaf.action_origin_ids, "action origin")?;
    if entry.requirement_origin_span != leaf.requirement_origin.span
        || entry.source_branch_id != branch_ids
        || entry.action_origin_id != action_ids
        || entry.required_profile_ids != leaf.required_profile_ids
    {
        return Err(format!(
            "generic interaction mapping identity differs for source candidate {}",
            leaf.source_candidate_fingerprint
        ));
    }
    let binding = &candidate.requirement_binding;
    if !entry.semantic_evidence.iter().any(|evidence| {
        evidence.source_path == binding.source_path
            && evidence.source_sha256 == binding.source_sha256
            && evidence.branch_id == binding.branch_id
            && evidence.span_start_line == binding.span_start_line
            && evidence.span_end_line == binding.span_end_line
            && evidence.source_excerpt_sha256 == binding.source_excerpt_sha256
    }) {
        return Err(format!(
            "generic interaction mapping has no matching semantic evidence for source candidate {}",
            leaf.source_candidate_fingerprint
        ));
    }
    Ok(())
}

pub(super) fn validate_semantics(entry: &GenericInteractionMappingEntry) -> Result<(), String> {
    if !KUC_INTERACTIONS.contains(&entry.interaction.as_str()) {
        return Err(format!(
            "generic interaction mapping has unsupported KUC interaction: {}",
            entry.interaction
        ));
    }
    if !EFFECT_KINDS.contains(&entry.declared_effect_kind.as_str()) {
        return Err(format!(
            "generic interaction mapping has unsupported effect kind: {}",
            entry.declared_effect_kind
        ));
    }
    if !is_canonical_profile_set_list(&entry.required_profile_ids)
        || entry.visible_path.is_empty()
        || entry.state_preconditions.is_empty()
        || has_invalid_text(&entry.visible_path)
        || has_invalid_text(&entry.state_preconditions)
        || [
            entry.kuc_component.as_str(),
            entry.kle_public_show_signature.as_str(),
            entry.kle_opaque_transit.as_str(),
        ]
        .into_iter()
        .any(is_invalid_text)
    {
        return Err("generic interaction mapping has unresolved semantic fields".into());
    }
    Ok(())
}

fn singleton<'a>(values: &'a [String], name: &str) -> Result<&'a str, String> {
    match values {
        [value] if !value.trim().is_empty() => Ok(value),
        _ => Err(format!("source candidate does not have exactly one {name}")),
    }
}

fn has_invalid_text(values: &[String]) -> bool {
    values.iter().any(|value| is_invalid_text(value))
}

fn is_invalid_text(value: &str) -> bool {
    value.trim().is_empty()
        || is_placeholder(value)
        || value.trim_start().starts_with("unresolved:")
}
