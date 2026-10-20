use super::super::GeneratedLeafJoinArtifact;
use super::fixtures;

#[test]
fn branch_referencing_missing_source_is_rejected() {
    let (source, mut branches, actions) = fixtures::inputs();
    branches.branches[0].file = "src/other.rs".to_string();
    let result =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions);
    assert!(result.is_err());
}

#[test]
fn valid_inputs_with_no_matching_action_produce_empty_candidates() -> Result<(), String> {
    let (source, branches, mut actions) = fixtures::inputs();
    actions.actions[0].forward_route = vec!["provisional_forward_route;file=src/other.rs".into()];
    let artifact =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions)?;
    assert!(artifact.candidates.is_empty());
    Ok(())
}
