use super::super::branch_catalog::materialize_branch_catalog;
use super::super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::super::source_action_binding::SourceActionBindingReport;
use super::super::source_action_binding::tests::fixture;
use super::super::structured_leaf_candidates::StructuredLeafCandidatesReport;
use super::*;

fn fixture_evidence() -> Result<(CanonicalLeafBindingEvidence, BranchCatalogArtifact), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches =
        materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())?;
    let t3 = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let structured =
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &t3)?;
    Ok((
        CanonicalLeafBindingEvidence {
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
fn projects_each_structured_action_variant_to_one_origin_id() -> Result<(), String> {
    let (evidence, branches) = fixture_evidence()?;
    let leafs = evidence.project_leafs(&branches)?;

    assert_eq!(leafs.len(), evidence.structured.candidates.len());
    for candidate in &evidence.structured.candidates {
        let leaf = leafs
            .iter()
            .find(|leaf| leaf.source_candidate_fingerprint == candidate.identity_fingerprint)
            .ok_or_else(|| "projected leaf is missing candidate provenance".to_string())?;
        assert_eq!(
            leaf.action_origin_ids,
            vec![format!(
                "katana:AppAction::{}",
                candidate.route.action_variant
            )]
        );
    }
    Ok(())
}

#[test]
fn rejects_empty_or_invalid_action_variant() -> Result<(), String> {
    for action_variant in ["", "not-valid"] {
        let (mut evidence, branches) = fixture_evidence()?;
        evidence.structured.candidates[0].route.action_variant = action_variant.into();

        assert!(matches!(
            evidence.project_leafs(&branches),
            Err(error) if error.contains("not a Rust identifier")
        ));
    }
    Ok(())
}

#[test]
fn rejects_changed_source_candidate_provenance() -> Result<(), String> {
    let (evidence, branches) = fixture_evidence()?;
    let mut leafs = evidence.project_leafs(&branches)?;
    leafs[0].source_branches.push("changed-provenance".into());

    assert!(matches!(
        evidence.validate_projection(&branches, &leafs),
        Err(error) if error.contains("immutable source provenance")
    ));
    Ok(())
}

#[test]
fn rejects_duplicate_leaf_provenance_with_a_missing_candidate_at_equal_population()
-> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches =
        materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())?;
    let t3 = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let mut structured =
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &t3)?;
    let mut second = structured.candidates[0].clone();
    second.identity_fingerprint = "distinct-candidate".into();
    structured.candidates.push(second);
    let evidence = CanonicalLeafBindingEvidence {
        inventory: RequirementSourceInventoryReport {
            requirements_sha256: "fixture".into(),
            rows: vec![],
            entries: vec![],
        },
        t2: facts.requirements,
        t3,
        structured,
    };
    let mut leafs = evidence.project_leafs(&branches)?;
    leafs[1].source_candidate_fingerprint = leafs[0].source_candidate_fingerprint.clone();
    assert!(matches!(
        evidence.validate_projection(&branches, &leafs),
        Err(error) if error.contains("duplicate candidate provenance")
    ));
    Ok(())
}
