use super::super::branch_catalog::materialize_branch_catalog;
use super::super::source_action_binding::tests::fixture;
use super::*;

fn current_binding() -> Result<
    (
        CanonicalLeafBindingEvidence,
        SourceClosureArtifact,
        BranchCatalogArtifact,
    ),
    String,
> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches =
        materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())?;
    let binding = CanonicalLeafBindingEvidence::build(&facts.source, &branches, &facts.state)?;
    Ok((binding, facts.source, branches))
}

#[test]
fn current_binding_roundtrips_and_validates() -> Result<(), String> {
    let (binding, source, branches) = current_binding()?;
    let encoded = serde_json::to_vec(&binding).map_err(|error| error.to_string())?;
    let decoded: CanonicalLeafBindingEvidence =
        serde_json::from_slice(&encoded).map_err(|error| error.to_string())?;

    assert_eq!(decoded.source_analysis_revision, SOURCE_ANALYSIS_REVISION);
    decoded.validate(&source, &branches)
}

#[test]
fn missing_source_analysis_revision_rejects_deserialization() -> Result<(), String> {
    let (binding, _, _) = current_binding()?;
    let mut value = serde_json::to_value(binding).map_err(|error| error.to_string())?;
    value
        .as_object_mut()
        .ok_or_else(|| "canonical binding must serialize as an object".to_string())?
        .remove("source_analysis_revision");

    assert!(serde_json::from_value::<CanonicalLeafBindingEvidence>(value).is_err());
    Ok(())
}

#[test]
fn changed_source_analysis_revision_rejects_before_layer_validation() -> Result<(), String> {
    let (binding, source, branches) = current_binding()?;
    let mut value = serde_json::to_value(binding).map_err(|error| error.to_string())?;
    value["source_analysis_revision"] = serde_json::json!("kle.source-analysis.v1");
    let changed: CanonicalLeafBindingEvidence =
        serde_json::from_value(value).map_err(|error| error.to_string())?;

    assert!(matches!(
        changed.validate(&source, &branches),
        Err(error) if error.contains("unsupported source analysis revision")
    ));
    Ok(())
}
