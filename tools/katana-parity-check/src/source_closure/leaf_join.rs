mod candidate_identity;
mod input_validation;
mod unresolved_facets;

use super::model::{
    ActionOrigin, ActionOriginsArtifact, BranchCatalogArtifact, BranchRecord,
    KATANA_ADOPTION_ISSUE_URL, KatanaHostE2eRequirement, KatanaHostE2eState, LeafManifestArtifact,
    LeafStatusRecord, RequirementOrigin, SourceClosureArtifact,
};

#[derive(Clone, Debug)]
pub(crate) struct GeneratedLeafJoinCandidate {
    pub leaf: LeafStatusRecord,
    pub identity_fingerprint: String,
    pub candidate_fingerprint: String,
    pub provenance: Vec<String>,
    pub unresolved_evidence: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct GeneratedLeafJoinArtifact {
    pub candidates: Vec<GeneratedLeafJoinCandidate>,
}

impl GeneratedLeafJoinArtifact {
    pub(crate) fn build_leaf_join_candidates(
        source: &SourceClosureArtifact,
        branches: &BranchCatalogArtifact,
        actions: &ActionOriginsArtifact,
    ) -> Result<Self, String> {
        input_validation::validate(source, branches, actions)?;
        let mut candidates = Vec::new();
        for branch in &branches.branches {
            for action in &actions.actions {
                if !route_mentions_file(action, &branch.file) {
                    continue;
                }
                if let Some(candidate) = build_candidate(source, branches, actions, branch, action)
                {
                    candidates.push(candidate);
                }
            }
        }
        candidates.sort_by(|left, right| {
            left.leaf
                .leaf_id
                .cmp(&right.leaf.leaf_id)
                .then(left.identity_fingerprint.cmp(&right.identity_fingerprint))
        });
        Ok(Self { candidates })
    }

    pub(crate) fn as_leaf_manifest(
        &self,
        root: super::ManifestRoot,
        canonical_binding: super::canonical_leaf_binding::CanonicalLeafBindingEvidence,
    ) -> LeafManifestArtifact {
        LeafManifestArtifact {
            root,
            canonical_binding,
            leafs: self
                .candidates
                .iter()
                .map(|candidate| candidate.leaf.clone())
                .collect(),
        }
    }
}

fn build_candidate(
    source: &SourceClosureArtifact,
    branches: &BranchCatalogArtifact,
    actions: &ActionOriginsArtifact,
    branch: &BranchRecord,
    action: &ActionOrigin,
) -> Option<GeneratedLeafJoinCandidate> {
    let source_file = source.files.iter().find(|file| file.path == branch.file)?;
    let identity_fingerprint = candidate_identity::identity_fingerprint(branch, action);
    let candidate_fingerprint = candidate_identity::candidate_fingerprint(
        &identity_fingerprint,
        source_file,
        branch,
        action,
    );
    let unresolved_evidence = unresolved_facets::evidence(branches, actions, branch, action);

    let required_profile_ids = branch
        .active_profile_ids
        .iter()
        .filter(|profile| action.active_profile_ids.contains(profile))
        .cloned()
        .collect::<Vec<_>>();
    let leaf_id = format!("generated-leaf:{identity_fingerprint}");
    Some(GeneratedLeafJoinCandidate {
        leaf: LeafStatusRecord {
            leaf_id,
            source_candidate_fingerprint: identity_fingerprint.clone(),
            requirement_origin: RequirementOrigin {
                kind: "generated_source_closure".to_string(),
                span: format!(
                    "{}:{}-{}",
                    branch.file, branch.span.start_line, branch.span.end_line
                ),
            },
            source_branches: vec![branch.branch_id.clone()],
            action_origin_ids: vec![action.action_id.clone()],
            required_profile_ids,
            visible_path: vec!["unresolved:visible-path".to_string()],
            state_preconditions: vec!["unresolved:state-preconditions".to_string()],
            kuc_component: "unresolved:kuc_component".to_string(),
            kle_public_show_signature: "unresolved:kle_public_show_signature".to_string(),
            kle_opaque_transit: "unresolved:kle_opaque_transit".to_string(),
            declared_effect_kind: "unresolved:actual_host_effect".to_string(),
            host_e2e: KatanaHostE2eRequirement {
                state: KatanaHostE2eState::DownstreamRequired,
                adoption_issue_url: KATANA_ADOPTION_ISSUE_URL.to_string(),
            },
            execution_id: None,
            storybook_stage_id: "unresolved:storybook_stage".to_string(),
            status: "candidate".to_string(),
        },
        identity_fingerprint,
        candidate_fingerprint: candidate_fingerprint.clone(),
        provenance: candidate_identity::provenance(
            source_file,
            branch,
            action,
            &candidate_fingerprint,
        ),
        unresolved_evidence,
    })
}

fn route_mentions_file(action: &ActionOrigin, file: &str) -> bool {
    action.forward_route.iter().any(|route| {
        route.split(';').any(|field| {
            field
                .strip_prefix("file=")
                .or_else(|| field.strip_prefix("source_file="))
                == Some(file)
        })
    })
}

#[cfg(test)]
#[path = "leaf_join_tests.rs"]
mod tests;
