use super::super::branch_catalog::materialize_branch_catalog;
use super::super::canonical_leaf_binding::SOURCE_ANALYSIS_REVISION;
use super::super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::super::source_action_binding::SourceActionBindingReport;
use super::super::source_action_binding::tests::fixture;
use super::super::structured_leaf_candidates::StructuredLeafCandidatesReport;
use super::*;

fn fixture_inputs() -> Result<(CanonicalLeafBindingEvidence, BranchCatalogArtifact), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let mut branches =
        materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())?;
    for branch in &mut branches.branches {
        branch.active_profile_ids = vec![
            "macos-latest".into(),
            "windows-latest".into(),
            "ubuntu-latest".into(),
        ];
    }
    let t3 = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let structured =
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &t3)?;
    Ok((
        CanonicalLeafBindingEvidence {
            source_analysis_revision: SOURCE_ANALYSIS_REVISION.into(),
            inventory: RequirementSourceInventoryReport {
                requirements_sha256: "fixture".into(),
                rows: vec![],
                entries: vec![],
            },
            t2: facts.requirements,
            t3,
            structured,
        },
        branches,
    ))
}

#[test]
fn catalog_covers_every_candidate_with_immutable_evidence() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let catalog = evidence.annotation_catalog(&branches)?;
    let output = serde_json::to_value(&catalog).map_err(|error| error.to_string())?;
    let entries = output["entries"]
        .as_array()
        .ok_or_else(|| "catalog entries must be an array".to_string())?;

    assert_eq!(output["schema"], SCHEMA);
    assert_eq!(entries.len(), evidence.structured.candidates.len());
    assert!(entries.iter().all(|entry| {
        entry["source_candidate_fingerprint"]
            .as_str()
            .is_some_and(|value| value.starts_with("sha256:"))
            && entry["source_sha256"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
            && entry["branch_excerpt_sha256"]
                .as_str()
                .is_some_and(|value| !value.is_empty())
    }));
    Ok(())
}
