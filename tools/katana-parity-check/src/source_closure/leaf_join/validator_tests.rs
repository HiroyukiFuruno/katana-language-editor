use super::super::super::ArtifactValidationMode;
use super::super::super::leaf_validation;
use super::fixtures;

#[test]
fn generated_candidate_manifest_fails_leaf_validator_closed() -> Result<(), String> {
    let artifact = fixtures::build()?;
    let manifest = artifact.as_leaf_manifest(fixtures::root(), fixtures::canonical_binding()?);
    let (_, branches, actions) = fixtures::inputs();
    let mut errors = Vec::new();
    let leaf_ids = leaf_validation::validate(
        &actions,
        &branches,
        &manifest,
        ArtifactValidationMode::Full,
        &mut errors,
    );
    assert_eq!(leaf_ids.len(), 1);
    assert!(errors.iter().any(|error| error.contains("invalid status")));
    assert!(
        errors
            .iter()
            .any(|error| error.contains("unresolved action origins"))
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("unclassified branches"))
    );
    Ok(())
}

#[test]
fn unresolved_join_facets_cannot_be_promoted_to_passing() -> Result<(), String> {
    let artifact = fixtures::build()?;
    let mut manifest = artifact.as_leaf_manifest(fixtures::root(), fixtures::canonical_binding()?);
    manifest.leafs[0].status = "passing".to_string();
    let (_, branches, actions) = fixtures::inputs();
    let mut errors = Vec::new();
    leaf_validation::validate(
        &actions,
        &branches,
        &manifest,
        ArtifactValidationMode::Full,
        &mut errors,
    );
    assert!(
        errors
            .iter()
            .any(|error| error.contains("invalid status/fields"))
    );
    Ok(())
}
