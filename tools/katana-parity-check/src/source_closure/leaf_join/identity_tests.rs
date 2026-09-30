use super::super::GeneratedLeafJoinArtifact;
use super::fixtures;

#[test]
fn candidate_identity_and_fingerprint_are_deterministic() -> Result<(), String> {
    let first = fixtures::build()?;
    let second = fixtures::build()?;
    assert_eq!(first.candidates.len(), second.candidates.len());
    assert_eq!(
        first.candidates[0].leaf.leaf_id,
        second.candidates[0].leaf.leaf_id
    );
    assert_eq!(
        first.candidates[0].identity_fingerprint,
        second.candidates[0].identity_fingerprint
    );
    assert_eq!(
        first.candidates[0].candidate_fingerprint,
        second.candidates[0].candidate_fingerprint
    );
    assert_eq!(
        first.candidates[0].provenance,
        second.candidates[0].provenance
    );
    assert_eq!(
        first.candidates[0].unresolved_evidence,
        second.candidates[0].unresolved_evidence
    );
    assert_eq!(first.candidates.len(), 1);
    assert!(
        first.candidates[0]
            .provenance
            .iter()
            .any(|fact| { fact.starts_with("candidate-fingerprint:sha256:") })
    );
    Ok(())
}

#[test]
fn source_fact_mutation_changes_candidate_fingerprint() -> Result<(), String> {
    let first = fixtures::build()?;
    let (mut source, branches, actions) = fixtures::inputs();
    source.files[0].sha256 = "sha256:file-mutated".to_string();
    let second =
        GeneratedLeafJoinArtifact::build_leaf_join_candidates(&source, &branches, &actions)?;
    assert_eq!(
        first.candidates[0].identity_fingerprint, second.candidates[0].identity_fingerprint,
        "identity is source branch/action identity and must remain stable for source hash-only changes"
    );
    assert_ne!(
        first.candidates[0].candidate_fingerprint,
        second.candidates[0].candidate_fingerprint
    );
    assert_ne!(
        first.candidates[0].provenance,
        second.candidates[0].provenance
    );
    Ok(())
}
