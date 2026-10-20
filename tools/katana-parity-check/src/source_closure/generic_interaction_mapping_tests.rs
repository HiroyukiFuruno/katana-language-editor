use super::super::branch_catalog::materialize_branch_catalog;
use super::super::canonical_leaf_binding::SOURCE_ANALYSIS_REVISION;
use super::super::requirement_source_inventory::RequirementSourceInventoryReport;
use super::super::source_action_binding::SourceActionBindingReport;
use super::super::source_action_binding::tests::fixture;
use super::super::structured_leaf_candidates::StructuredLeafCandidatesReport;
use super::*;

fn provenance() -> ConsumerPlanProvenance {
    ConsumerPlanProvenance {
        run_id: "test-run".into(),
        source_closure_sha256: format!("sha256:{}", "a".repeat(64)),
        profile_fingerprint: format!("sha256:{}", "b".repeat(64)),
    }
}

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

fn mapping_for(
    evidence: &CanonicalLeafBindingEvidence,
    branches: &BranchCatalogArtifact,
) -> Result<GenericInteractionMapping, String> {
    let leafs = evidence.project_leafs(branches)?;
    Ok(GenericInteractionMapping {
        schema: SCHEMA.into(),
        mappings: evidence
            .structured
            .candidates
            .iter()
            .map(|candidate| {
                let leaf = leafs
                    .iter()
                    .find(|leaf| {
                        leaf.source_candidate_fingerprint == candidate.identity_fingerprint
                    })
                    .ok_or_else(|| "fixture candidate must have a projected leaf".to_string())?;
                Ok(GenericInteractionMappingEntry {
                    source_candidate_fingerprint: leaf.source_candidate_fingerprint.clone(),
                    requirement_origin_span: leaf.requirement_origin.span.clone(),
                    source_branch_id: leaf.source_branches[0].clone(),
                    action_origin_id: leaf.action_origin_ids[0].clone(),
                    required_profile_ids: vec![
                        "macos-latest".into(),
                        "windows-latest".into(),
                        "ubuntu-latest".into(),
                    ],
                    interaction: "toolbar_activation".into(),
                    visible_path: vec!["editor".into(), "toolbar".into()],
                    state_preconditions: vec!["editable".into()],
                    kuc_component: "TextCommandSurface".into(),
                    kle_public_show_signature: "EguiLanguageEditor::show".into(),
                    kle_opaque_transit: "opaque-target".into(),
                    declared_effect_kind: "in_process_host_effect".into(),
                    semantic_evidence: vec![SemanticEvidence {
                        source_path: candidate.requirement_binding.source_path.clone(),
                        source_sha256: candidate.requirement_binding.source_sha256.clone(),
                        branch_id: candidate.requirement_binding.branch_id.clone(),
                        span_start_line: candidate.requirement_binding.span_start_line,
                        span_end_line: candidate.requirement_binding.span_end_line,
                        source_excerpt_sha256: candidate
                            .requirement_binding
                            .source_excerpt_sha256
                            .clone(),
                    }],
                })
            })
            .collect::<Result<Vec<_>, String>>()?,
    })
}

#[test]
fn enriches_only_a_total_identity_matched_mapping() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mapping = mapping_for(&evidence, &branches)?;
    let leafs = evidence.enrich_leafs(&branches, &mapping)?;

    assert_eq!(leafs.len(), evidence.structured.candidates.len());
    assert!(leafs.iter().all(|leaf| {
        leaf.status == "mapped_pending_evidence"
            && leaf.storybook_stage_id.starts_with("kuc-stage:")
            && leaf.kuc_component == "TextCommandSurface"
    }));
    Ok(())
}

#[test]
fn consumer_plan_preserves_each_verified_leaf_interaction() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mut mapping = mapping_for(&evidence, &branches)?;
    for (index, entry) in mapping.mappings.iter_mut().enumerate() {
        entry.interaction = if index % 2 == 0 {
            "selection"
        } else {
            "search"
        }
        .into();
    }
    for profile in super::super::operational_cli::PROFILE_IDS {
        let plan = mapping.consumer_plan(&evidence, &branches, profile, provenance())?;
        assert_eq!(plan.schema, "kle.consumer-artifact-plan.v1");
        assert_eq!(plan.profile_id, profile);
        assert_eq!(plan.stages.len(), mapping.mappings.len());
        let wire = serde_json::to_value(&plan).map_err(|error| error.to_string())?;
        assert_eq!(wire["run_id"], "test-run");
        assert_eq!(
            wire["source_closure_sha256"],
            provenance().source_closure_sha256
        );
        assert_eq!(
            wire["profile_fingerprint"],
            provenance().profile_fingerprint
        );
        for stage in plan.stages {
            let entry = mapping
                .mappings
                .iter()
                .find(|entry| {
                    stage.leaf_id
                        == format!("source-candidate:{}", entry.source_candidate_fingerprint)
                })
                .ok_or_else(|| "plan stage has no mapping".to_string())?;
            assert_eq!(stage.interaction, entry.interaction);
        }
    }
    Ok(())
}

#[test]
fn consumer_plan_rejects_unusable_run_provenance() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mapping = mapping_for(&evidence, &branches)?;
    for field in ["run", "source", "profile"] {
        let mut facts = provenance();
        match field {
            "run" => facts.run_id = "invalid run".into(),
            "source" => facts.source_closure_sha256 = "sha256:short".into(),
            _ => facts.profile_fingerprint = format!("sha256:{}", "A".repeat(64)),
        }
        assert!(
            matches!(mapping.consumer_plan(&evidence, &branches, "macos-latest", facts), Err(error) if error.contains("provenance"))
        );
    }
    Ok(())
}

#[test]
fn consumer_plan_rejects_incomplete_mapping_and_unknown_profile() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mut mapping = mapping_for(&evidence, &branches)?;
    assert!(
        matches!(mapping.consumer_plan(&evidence, &branches, "other", provenance()), Err(error) if error.contains("unsupported"))
    );
    mapping.mappings.pop();
    assert!(
        matches!(mapping.consumer_plan(&evidence, &branches, "macos-latest", provenance()), Err(error) if error.contains("not total"))
    );
    Ok(())
}

#[test]
fn consumer_plan_revalidates_semantics_before_emitting_stages() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    for field in ["interaction", "effect", "component", "evidence"] {
        let mut mapping = mapping_for(&evidence, &branches)?;
        let entry = mapping
            .mappings
            .first_mut()
            .ok_or_else(|| "fixture has no mappings".to_string())?;
        match field {
            "interaction" => entry.interaction = "unknown".into(),
            "effect" => entry.declared_effect_kind = "unknown".into(),
            "component" => entry.kuc_component = "unresolved:component".into(),
            _ => entry.semantic_evidence.clear(),
        }
        assert!(
            mapping
                .consumer_plan(&evidence, &branches, "macos-latest", provenance())
                .is_err()
        );
    }
    Ok(())
}

#[test]
fn rejects_missing_duplicate_and_unknown_mappings() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mut missing = mapping_for(&evidence, &branches)?;
    missing.mappings.pop();
    assert!(matches!(
        evidence.enrich_leafs(&branches, &missing),
        Err(error) if error.contains("not total")
    ));

    let mut duplicate = mapping_for(&evidence, &branches)?;
    duplicate.mappings.push(duplicate.mappings[0].clone());
    assert!(matches!(
        evidence.enrich_leafs(&branches, &duplicate),
        Err(error) if error.contains("duplicate")
    ));

    let mut unknown = mapping_for(&evidence, &branches)?;
    unknown.mappings[0].source_candidate_fingerprint = "unknown".into();
    assert!(matches!(
        evidence.enrich_leafs(&branches, &unknown),
        Err(error) if error.contains("missing source candidate")
    ));
    Ok(())
}

#[test]
fn rejects_semantic_drift_and_unresolved_values() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mut drift = mapping_for(&evidence, &branches)?;
    drift.mappings[0].action_origin_id = "katana:AppAction::Other".into();
    assert!(matches!(
        evidence.enrich_leafs(&branches, &drift),
        Err(error) if error.contains("identity differs")
    ));

    let mut unresolved = mapping_for(&evidence, &branches)?;
    unresolved.mappings[0].kuc_component = "unresolved:component".into();
    assert!(matches!(
        evidence.enrich_leafs(&branches, &unresolved),
        Err(error) if error.contains("unresolved semantic")
    ));
    Ok(())
}

#[test]
fn rejects_a_mapping_without_source_matched_semantic_evidence() -> Result<(), String> {
    let (evidence, branches) = fixture_inputs()?;
    let mut mapping = mapping_for(&evidence, &branches)?;
    mapping.mappings[0].semantic_evidence[0].source_sha256 = "sha256:other".into();

    assert!(matches!(
        evidence.enrich_leafs(&branches, &mapping),
        Err(error) if error.contains("no matching semantic evidence")
    ));
    Ok(())
}
