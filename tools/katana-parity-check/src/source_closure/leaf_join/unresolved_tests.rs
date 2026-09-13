use super::fixtures;

#[test]
fn every_missing_join_facet_is_retained_as_unresolved_evidence() -> Result<(), String> {
    let artifact = fixtures::build()?;
    let candidate = &artifact.candidates[0];
    let evidence = &candidate.unresolved_evidence;
    assert!(evidence.iter().any(|fact| fact.contains("kuc component")));
    assert!(evidence.iter().any(|fact| fact.contains("KLE public-show")));
    assert!(
        evidence
            .iter()
            .any(|fact| fact.contains("actual host effect"))
    );
    assert!(evidence.iter().any(|fact| fact.contains("Storybook stage")));
    Ok(())
}

#[test]
fn unclassified_source_facts_cannot_be_completed() -> Result<(), String> {
    let artifact = fixtures::build()?;
    let candidate = &artifact.candidates[0];
    assert_eq!(candidate.leaf.status, "candidate");
    assert!(
        candidate
            .unresolved_evidence
            .iter()
            .any(|fact| { fact.contains("source branch is unclassified") })
    );
    assert!(
        candidate
            .unresolved_evidence
            .iter()
            .any(|fact| { fact.contains("action origin is unclassified or unresolved") })
    );
    Ok(())
}
