use super::super::branch_catalog::materialize_branch_catalog;
use super::super::requirement_binding::RequirementBindingResult;
use super::super::source_action_binding::SourceActionBindingReport;
use super::super::source_requirement_alias_ledger::AliasEntry;
use super::*;

use super::super::source_action_binding::tests::fixture;

fn entries() -> [AliasEntry; 1] {
    [AliasEntry {
        requirement_id: "editor.one".into(),
        short_reference: "lib.rs".into(),
        source_path: "crates/katana-ui/src/editor.rs".into(),
    }]
}

fn branches(
    facts: &fixture::FixtureFacts,
) -> Result<super::super::model::BranchCatalogArtifact, String> {
    materialize_branch_catalog(facts.root.clone(), &facts.state, &[], facts._fixture.path())
        .map_err(|error| error.to_string())
}

#[test]
fn joins_exact_binding_ids_and_retains_diagnostic_evidence() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let report = StructuredLeafCandidatesReport::build(
        &facts.source,
        &branches,
        &facts.requirements,
        &actions,
    )?;
    assert_eq!(report.candidates.len(), 1);
    assert_eq!(
        report.unmatched_requirement_binding_ids,
        actions.unmatched_requirement_binding_ids
    );
    assert_eq!(report.requirement_unresolved, facts.requirements.unresolved);
    assert_eq!(report.unbound_branches, facts.requirements.unbound_branches);
    assert_eq!(
        serde_json::to_vec(&report.source_action_unresolved).map_err(|error| error.to_string())?,
        serde_json::to_vec(&actions.unresolved).map_err(|error| error.to_string())?
    );
    Ok(())
}

#[test]
fn construction_outside_branch_span_is_preserved_as_zero_reference_route() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let mut state = facts.state;
    state.action_constructions[0].span = "katana:crates/katana-ui/src/editor.rs:99:1-99:8".into();
    let actions = SourceActionBindingReport::build(&state, &facts.source, &facts.requirements)?;
    let report = StructuredLeafCandidatesReport::build(
        &facts.source,
        &branches,
        &facts.requirements,
        &actions,
    )?;
    assert!(report.candidates.is_empty());
    assert!(!report.zero_reference_routes.is_empty());
    Ok(())
}

#[test]
fn forged_existing_binding_outside_branch_span_is_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let mut actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let binding_id = actions.candidates[0].requirement_binding_ids[0].clone();
    actions.candidates[0].construction.span =
        "katana:crates/katana-ui/src/editor.rs:99:1-99:8".into();
    actions.candidates[0].requirement_binding_ids = vec![binding_id];
    actions.fingerprint = actions.fingerprint_value()?;
    assert!(matches!(
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &actions),
        Err(error) if error.contains("outside requirement binding branch span")
    ));
    Ok(())
}

#[test]
fn tampered_report_fingerprint_is_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let mut actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    actions.fingerprint = "tampered".into();
    assert!(
        StructuredLeafCandidatesReport::build(
            &facts.source,
            &branches,
            &facts.requirements,
            &actions
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn rehashed_t3_fact_with_wrong_span_path_is_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let mut actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    actions.candidates[0].definition.span = "katana:other.rs:1:1-1:2".into();
    actions.fingerprint = actions.fingerprint_value()?;
    assert!(
        StructuredLeafCandidatesReport::build(
            &facts.source,
            &branches,
            &facts.requirements,
            &actions
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn missing_and_duplicate_binding_references_are_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let mut missing = actions.clone();
    missing.candidates[0].requirement_binding_ids = vec!["REQ:src/app.rs:missing".into()];
    missing.fingerprint = missing.fingerprint_value()?;
    assert!(matches!(
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &missing),
        Err(error) if error.contains("unknown requirement binding reference")
    ));
    let mut duplicate = actions;
    let id = duplicate.candidates[0].requirement_binding_ids[0].clone();
    duplicate.candidates[0].requirement_binding_ids.push(id);
    duplicate.fingerprint = duplicate.fingerprint_value()?;
    assert!(matches!(
        StructuredLeafCandidatesReport::build(&facts.source, &branches, &facts.requirements, &duplicate),
        Err(error) if error.contains("duplicate requirement binding reference")
    ));
    Ok(())
}

#[test]
fn source_and_branch_fact_mismatches_are_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let mut source = facts.source.clone();
    source.files[0].sha256 = "wrong".into();
    assert!(
        StructuredLeafCandidatesReport::build(&source, &branches, &facts.requirements, &actions)
            .is_err()
    );
    let mut changed_branches = branches;
    changed_branches.branches[0].span.start_line += 1;
    assert!(
        StructuredLeafCandidatesReport::build(
            &facts.source,
            &changed_branches,
            &facts.requirements,
            &actions
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn reordered_t2_and_t3_rebuilds_have_stable_identity() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let first_actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let first = StructuredLeafCandidatesReport::build(
        &facts.source,
        &branches,
        &facts.requirements,
        &first_actions,
    )?;
    let mut entries = entries().to_vec();
    entries.reverse();
    let mut reordered_branches = branches;
    reordered_branches.branches.reverse();
    let requirements =
        RequirementBindingResult::build(&entries, &facts.source, &reordered_branches)?;
    let mut state = facts.state;
    state.action_constructions.reverse();
    state.dispatch_arms.reverse();
    let actions = SourceActionBindingReport::build(&state, &facts.source, &requirements)?;
    let second = StructuredLeafCandidatesReport::build(
        &facts.source,
        &reordered_branches,
        &requirements,
        &actions,
    )?;
    assert_eq!(first.fingerprint, second.fingerprint);
    assert_eq!(
        first.candidates[0].identity_fingerprint,
        second.candidates[0].identity_fingerprint
    );
    Ok(())
}

#[test]
fn all_root_fields_are_checked() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    let actions =
        SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
    let rejects = |mutate: fn(&mut super::super::model::ManifestRoot)| {
        let mut changed = branches.clone();
        mutate(&mut changed.root);
        assert!(
            StructuredLeafCandidatesReport::build(
                &facts.source,
                &changed,
                &facts.requirements,
                &actions
            )
            .is_err()
        );
    };
    rejects(|root| root.schema_version = "other".into());
    rejects(|root| root.katana_revision = "other".into());
    rejects(|root| root.katana_tree_fingerprint = "other".into());
    rejects(|root| root.katana_external_ui_fingerprint = "other".into());
    rejects(|root| root.user_mandated_extensions_fingerprint = "other".into());
    rejects(|root| root.source_universe_fingerprint = "other".into());
    rejects(|root| root.requirement_source_aliases_fingerprint = "other".into());
    rejects(|root| root.kle_tree_fingerprint = "other".into());
    rejects(|root| root.kuc_tree_fingerprint = "other".into());
    rejects(|root| root.release_profile_matrix_fingerprint = "other".into());
    rejects(|root| root.generator_fingerprint = "other".into());
    rejects(|root| root.generated_at_utc = "other".into());
    rejects(|root| root.static_leaf_count = Some(1));
    rejects(|root| root.expected_leaf_ids = Some(vec!["other".into()]));
    Ok(())
}

#[test]
fn rehashed_precision_overclaims_are_rejected() -> Result<(), String> {
    let facts = fixture::actual_fixture().map_err(|error| error.to_string())?;
    let branches = branches(&facts)?;
    for change_route in [false, true] {
        let mut actions =
            SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)?;
        if change_route {
            actions.candidates[0].branch_span_precision = "exact".into();
        } else {
            actions.branch_span_precision = "exact".into();
        }
        actions.fingerprint = actions.fingerprint_value()?;
        let result = StructuredLeafCandidatesReport::build(
            &facts.source,
            &branches,
            &facts.requirements,
            &actions,
        );
        assert!(matches!(result, Err(error) if error.contains("span precision is invalid")));
    }
    Ok(())
}
