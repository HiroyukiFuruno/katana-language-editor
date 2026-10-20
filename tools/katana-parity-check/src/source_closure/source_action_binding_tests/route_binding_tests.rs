#[test]
fn parsed_facts_produce_inside_join_and_outside_route_candidate()
-> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let report = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    assert!(report.candidates.len() >= 2);
    assert!(
        report
            .candidates
            .iter()
            .any(|candidate| !candidate.requirement_binding_ids.is_empty())
    );
    assert!(
        report
            .candidates
            .iter()
            .any(|candidate| candidate.requirement_binding_ids.is_empty())
    );
    assert!(report.candidates.iter().all(|candidate| {
        candidate.definition.path == "crates/katana-ui/src/app_action.rs"
            && candidate.construction.path == "crates/katana-ui/src/editor.rs"
            && candidate.dispatch.path == "crates/katana-ui/src/dispatch.rs"
            && candidate.dispatch_variant_pattern.path == "crates/katana-ui/src/dispatch.rs"
    }));
    let source_hashes = facts
        .source
        .files
        .iter()
        .map(|file| (file.path.as_str(), file.sha256.as_str()))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert!(report.candidates.iter().all(|candidate| {
        source_hashes.get(candidate.definition.path.as_str())
            == Some(&candidate.definition.sha256.as_str())
            && source_hashes.get(candidate.construction.path.as_str())
                == Some(&candidate.construction.sha256.as_str())
            && source_hashes.get(candidate.dispatch.path.as_str())
                == Some(&candidate.dispatch.sha256.as_str())
            && source_hashes.get(candidate.dispatch_variant_pattern.path.as_str())
                == Some(&candidate.dispatch_variant_pattern.sha256.as_str())
    }));
    assert!(
        report
            .unresolved
            .iter()
            .all(|entry| !entry.reason.contains("passing"))
    );
    assert_eq!(
        report.branch_span_precision,
        "line_level_only; same_line_nesting_not_proven"
    );
    let _: SourceActionBindingReport = serde_json::from_value(serde_json::to_value(&report)?)?;
    Ok(())
}

#[test]
fn route_fact_mutation_changes_report_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let FixtureFacts {
        _fixture: _,
        root: _,
        state,
        source,
        requirements,
    } = facts;
    let first = SourceActionBindingReport::build(&state, &source, &requirements)
        .map_err(std::io::Error::other)?;
    let mut changed_state = state;
    let dispatch = changed_state
        .dispatch_arms
        .first_mut()
        .ok_or("parsed fixture dispatch arm missing")?;
    dispatch.arm_span = dispatch.dispatch_span.clone();
    let changed = SourceActionBindingReport::build(&changed_state, &source, &requirements)
        .map_err(std::io::Error::other)?;
    assert_ne!(first.fingerprint, changed.fingerprint);
    Ok(())
}

#[test]
fn aliases_macros_and_dispatch_reasons_remain_explicitly_unresolved()
-> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut state = facts.state;
    state.action_constructions.push(ActionConstruction {
        file: "crates/katana-ui/src/editor.rs".into(),
        symbol: "emit".into(),
        span: "katana:crates/katana-ui/src/editor.rs:2:5-2:20".into(),
        enum_name: "AppAction".into(),
        variant: None,
        style: "alias".into(),
        unresolved_reason: Some("alias construction is unresolved".into()),
        input_origin_candidates: Vec::new(),
    });
    state.action_constructions.push(ActionConstruction {
        file: "crates/katana-ui/src/editor.rs".into(),
        symbol: "emit".into(),
        span: "katana:crates/katana-ui/src/editor.rs:2:5-2:20".into(),
        enum_name: "AppAction".into(),
        variant: None,
        style: "macro".into(),
        unresolved_reason: Some("macro construction is unresolved".into()),
        input_origin_candidates: Vec::new(),
    });
    let dispatch = state
        .dispatch_arms
        .first_mut()
        .ok_or("parsed fixture dispatch arm missing")?;
    dispatch
        .unresolved_reasons
        .push("dispatch handler resolution is unresolved".into());
    let report = SourceActionBindingReport::build(&state, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    assert!(report.unresolved.iter().any(|entry| {
        entry.kind == "unresolved_construction"
            && entry.reason == "alias construction is unresolved"
    }));
    assert!(report.unresolved.iter().any(|entry| {
        entry.kind == "unresolved_construction"
            && entry.reason == "macro construction is unresolved"
    }));
    assert!(report.unresolved.iter().any(|entry| {
        entry.kind == "unresolved_dispatch_arm"
            && entry.reason == "dispatch handler resolution is unresolved"
    }));
    Ok(())
}

#[test]
fn reversed_fact_order_has_identical_serialized_report() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let first = SourceActionBindingReport::build(&facts.state, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    let mut reversed = facts.state;
    reversed.action_definitions.reverse();
    reversed.action_constructions.reverse();
    reversed.dispatch_arms.reverse();
    reversed.dispatch_fallthroughs.reverse();
    reversed.unresolved_edges.reverse();
    let second = SourceActionBindingReport::build(&reversed, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn missing_duplicate_and_root_mismatch_fail_closed() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut missing_source = facts.source.clone();
    missing_source.files.clear();
    assert!(matches!(
        SourceActionBindingReport::build(&facts.state, &missing_source, &facts.requirements),
        Err(error) if error.contains("source path is absent from closure")
    ));

    let mut duplicate_source = facts.source.clone();
    let file = duplicate_source
        .files
        .first()
        .cloned()
        .ok_or("parsed fixture source file missing")?;
    duplicate_source.files.push(file);
    assert!(matches!(
        SourceActionBindingReport::build(&facts.state, &duplicate_source, &facts.requirements),
        Err(error) if error.contains("duplicate source file path")
    ));

    let mut mismatched_root = facts.root.clone();
    mismatched_root.katana_revision = "foreign".into();
    let mut mismatched_source = facts.source.clone();
    mismatched_source.root = mismatched_root;
    assert!(matches!(
        SourceActionBindingReport::build(&facts.state, &mismatched_source, &facts.requirements),
        Err(error) if error.contains("roots differ")
    ));
    Ok(())
}

#[test]
fn tampered_requirement_source_sha_or_fingerprint_cannot_join()
-> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut tampered_sha = facts.requirements.clone();
    tampered_sha.bindings[0].source_sha256 = "sha256:foreign".into();
    assert!(matches!(
        SourceActionBindingReport::build(&facts.state, &facts.source, &tampered_sha),
        Err(error) if error.contains("source SHA mismatch")
    ));

    let mut tampered_fingerprint = facts.requirements.clone();
    tampered_fingerprint.fingerprint = "sha256:foreign".into();
    assert!(matches!(
        SourceActionBindingReport::build(&facts.state, &facts.source, &tampered_fingerprint),
        Err(error) if error.contains("fingerprint mismatch")
    ));
    Ok(())
}

#[test]
fn dispatch_without_arm_is_retained_as_unresolved() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut state = facts.state;
    state.dispatch_arms.clear();
    let report = SourceActionBindingReport::build(&state, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    assert!(
        report
            .unresolved
            .iter()
            .any(|entry| entry.kind == "missing_dispatch_arm")
    );
    Ok(())
}

#[test]
fn ambiguous_definition_and_unrelated_variant_do_not_join_requirement()
-> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let FixtureFacts {
        _fixture: _,
        root: _,
        mut state,
        source,
        requirements,
    } = facts;
    let duplicate = state
        .action_definitions
        .first()
        .cloned()
        .ok_or("parsed fixture definition missing")?;
    state.action_definitions.push(duplicate);
    let report = SourceActionBindingReport::build(&state, &source, &requirements)
        .map_err(std::io::Error::other)?;
    assert!(
        report
            .unresolved
            .iter()
            .any(|entry| entry.kind == "ambiguous_definition")
    );
    assert!(report.candidates.iter().all(|candidate| {
        candidate.action_variant != "Bold" || candidate.requirement_binding_ids.is_empty()
    }));
    Ok(())
}
use super::super::super::scan_state::ActionConstruction;
use super::super::super::source_action_binding::SourceActionBindingReport;
use super::fixture::{FixtureFacts, actual_fixture};
