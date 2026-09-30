use super::super::model::{
    ActionOrigin, ActionOriginsArtifact, BranchCatalogArtifact, BranchRecord,
};

pub(super) fn evidence(
    branches: &BranchCatalogArtifact,
    actions: &ActionOriginsArtifact,
    branch: &BranchRecord,
    action: &ActionOrigin,
) -> Vec<String> {
    let mut evidence = vec![
        "generated unresolved: kuc component join is absent".to_string(),
        "generated unresolved: KLE public-show and opaque-transit join is absent".to_string(),
        "generated unresolved: actual host effect and execution join is absent".to_string(),
        "generated unresolved: Storybook stage join is absent".to_string(),
    ];
    if branches.unclassified_branch_ids.contains(&branch.branch_id) {
        evidence.push(format!(
            "generated unresolved: source branch is unclassified: {}",
            branch.branch_id
        ));
    }
    if action.origin_classification.trim().is_empty()
        || action.origin_classification == "unresolved"
        || actions
            .unresolved_action_origins
            .iter()
            .any(|entry| entry.starts_with(&action.action_id))
    {
        evidence.push(format!(
            "generated unresolved: action origin is unclassified or unresolved: {}",
            action.action_id
        ));
    }
    if action.kle_leafs.is_empty() {
        evidence.push(format!(
            "generated unresolved: action has no proven KLE leaf join: {}",
            action.action_id
        ));
    }
    evidence.sort();
    evidence.dedup();
    evidence
}
