use std::collections::{BTreeMap, BTreeSet};

use super::artifact_model::{BranchCatalogArtifact, LeafStatusRecord, RequirementOrigin};
use super::canonical_leaf_binding::CanonicalLeafBindingEvidence;
use super::model::{KATANA_ADOPTION_ISSUE_URL, KatanaHostE2eRequirement, KatanaHostE2eState};

pub(super) fn project(
    evidence: &CanonicalLeafBindingEvidence,
    branches: &BranchCatalogArtifact,
) -> Result<Vec<LeafStatusRecord>, String> {
    let mut leafs = Vec::new();
    for candidate in &evidence.structured.candidates {
        let branch = branches
            .branches
            .iter()
            .find(|branch| branch.branch_id == candidate.requirement_binding.branch_id)
            .ok_or_else(|| "structured candidate branch is absent from catalog".to_string())?;
        let action_origin_id = action_origin_id(&candidate.route.action_variant)?;
        leafs.push(LeafStatusRecord {
            leaf_id: format!("source-candidate:{}", candidate.identity_fingerprint),
            source_candidate_fingerprint: candidate.identity_fingerprint.clone(),
            requirement_origin: RequirementOrigin {
                kind: "structured_requirement_binding".into(),
                span: format!(
                    "{}:{}-{}",
                    candidate.requirement_binding.source_path,
                    candidate.requirement_binding.span_start_line,
                    candidate.requirement_binding.span_end_line
                ),
            },
            source_branches: vec![candidate.requirement_binding.branch_id.clone()],
            action_origin_ids: vec![action_origin_id],
            required_profile_ids: branch.active_profile_ids.clone(),
            visible_path: vec!["unresolved:visible-path".into()],
            state_preconditions: vec!["unresolved:state-preconditions".into()],
            kuc_component: "unresolved:kuc-component".into(),
            kle_public_show_signature: "unresolved:kle-public-show".into(),
            kle_opaque_transit: "unresolved:kle-opaque-transit".into(),
            declared_effect_kind: "unresolved:actual-host-effect".into(),
            host_e2e: KatanaHostE2eRequirement {
                state: KatanaHostE2eState::DownstreamRequired,
                adoption_issue_url: KATANA_ADOPTION_ISSUE_URL.into(),
            },
            execution_id: None,
            storybook_stage_id: "unresolved:storybook-stage".into(),
            status: "candidate".into(),
        });
    }
    leafs.sort_by(|left, right| left.leaf_id.cmp(&right.leaf_id));
    Ok(leafs)
}

pub(super) fn validate(
    evidence: &CanonicalLeafBindingEvidence,
    branches: &BranchCatalogArtifact,
    leafs: &[LeafStatusRecord],
) -> Result<(), String> {
    let mut expected = project(evidence, branches)?
        .into_iter()
        .map(|leaf| (leaf.source_candidate_fingerprint.clone(), leaf))
        .collect::<BTreeMap<_, _>>();
    if expected.len() != evidence.structured.candidates.len() {
        return Err("structured candidates have duplicate identities".into());
    }
    let mut seen = BTreeSet::new();
    for leaf in leafs {
        if !seen.insert(leaf.source_candidate_fingerprint.as_str()) {
            return Err("leaf projection has duplicate candidate provenance".into());
        }
        let candidate = expected
            .remove(&leaf.source_candidate_fingerprint)
            .ok_or_else(|| "leaf has no structured candidate provenance".to_string())?;
        if leaf.leaf_id != candidate.leaf_id
            || leaf.requirement_origin.kind != candidate.requirement_origin.kind
            || leaf.requirement_origin.span != candidate.requirement_origin.span
            || leaf.source_branches != candidate.source_branches
            || leaf.action_origin_ids != candidate.action_origin_ids
            || leaf.required_profile_ids != candidate.required_profile_ids
        {
            return Err(
                "leaf immutable source provenance differs from structured candidate".into(),
            );
        }
    }
    if expected.is_empty() {
        Ok(())
    } else {
        Err("structured candidates are missing from leaf projection".into())
    }
}

fn action_origin_id(action_variant: &str) -> Result<String, String> {
    if syn::parse_str::<syn::Ident>(action_variant).is_err() {
        return Err("structured candidate action variant is not a Rust identifier".into());
    }
    Ok(format!("katana:AppAction::{action_variant}"))
}

#[cfg(test)]
#[path = "canonical_leaf_projection_tests.rs"]
mod tests;
