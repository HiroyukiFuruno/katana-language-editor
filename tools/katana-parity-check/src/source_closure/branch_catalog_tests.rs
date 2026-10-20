use super::*;

#[test]
fn branch_catalog_records_cfg_if_and_match_edges_one_to_one() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "branch-catalog-kinds",
        &[(
            "src/lib.rs",
            "#[cfg(target_os = \"macos\")]\nfn gated() {}\nfn run(value: Option<u8>) { if value.is_some() { } match value { Some(inner) => {}, None => {} } }",
        )],
        "src/lib.rs",
    )?;
    let catalog = string_result(
        crate::source_closure::branch_catalog::materialize_branch_catalog(
            test_manifest_root(),
            &state,
            &test_profiles(),
            &root,
        ),
    )?;
    let branch_edge_count = state
        .edges
        .iter()
        .filter(|edge| crate::source_closure::branch_catalog::is_branch_kind(&edge.kind))
        .count();

    assert_eq!(catalog.branches.len(), branch_edge_count);
    for kind in ["cfg", "if", "match"] {
        assert!(
            catalog.branches.iter().any(|branch| branch.kind == kind),
            "missing {kind} branch"
        );
    }
    assert!(
        catalog
            .branches
            .iter()
            .any(|branch| { branch.kind == "cfg" && branch.condition == "target_os = \"macos\"" })
    );
    assert!(
        catalog
            .branches
            .iter()
            .any(|branch| { branch.kind == "if" && branch.condition == "value.is_some()" })
    );
    assert!(
        catalog
            .branches
            .iter()
            .any(|branch| { branch.kind == "match" && branch.condition == "Some(inner)" })
    );
    assert_eq!(
        catalog.unclassified_branch_ids.len(),
        catalog.branches.len(),
        "branch catalog slice must keep semantic outcomes unclassified"
    );
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn branch_catalog_enumerates_control_flow_with_exact_ast_spans() -> TestResult {
    let source = r#"
struct Item { enabled: bool }

fn run(value: Option<Item>, values: Vec<Option<Item>>, enabled: bool) -> Result<(), ()> {
    let japanese = "日本語🦀";
    if let Some(Item { enabled: true }) =
        value
        && enabled
    {
    } else {
    }
    match value {
        Some(Item { enabled: true }) if enabled => {},
        _ => {},
    }
    let Some(Item { enabled: true }) = value else { return Ok(()); };
    let _ = enabled && value.is_some();
    let _ = enabled || value.is_some();
    for item in values { let _ = item; }
    while enabled { break; }
    loop { continue; }
    let _ = value.ok_or(())?;
    return Ok(());
}
"#;
    let (state, _discovered, root) = scan_source_fixture(
        "branch-catalog-control-flow",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let catalog = string_result(
        crate::source_closure::branch_catalog::materialize_branch_catalog(
            test_manifest_root(),
            &state,
            &test_profiles(),
            &root,
        ),
    )?;

    for kind in [
        "if",
        "if_then",
        "if_else",
        "match",
        "match_guard",
        "match_body",
        "let_else",
        "let_else_body",
        "logical_and",
        "logical_or",
        "for",
        "for_body",
        "while",
        "while_body",
        "loop",
        "loop_body",
        "return",
        "break",
        "continue",
        "try",
    ] {
        assert!(
            catalog.branches.iter().any(|branch| branch.kind == kind),
            "missing {kind} branch"
        );
    }
    assert!(catalog.branches.iter().any(|branch| {
        branch.kind == "if" && branch.condition.contains("Some(Item { enabled: true })")
    }));
    assert!(catalog.branches.iter().any(|branch| {
        branch.kind == "match" && branch.condition == "Some(Item { enabled: true })"
    }));
    assert!(
        catalog
            .branches
            .iter()
            .all(|branch| { catalog.unclassified_branch_ids.contains(&branch.branch_id) })
    );

    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn branch_catalog_keeps_duplicate_source_spans_as_individual_unclassified_candidates() -> TestResult
{
    let catalog = catalog_for_source(
        "branch-catalog-duplicate-spans",
        "fn run(value: bool) { if value {} if value {} }",
    )?;
    let conditions = catalog
        .branches
        .iter()
        .filter(|branch| branch.kind == "if" && branch.condition == "value")
        .collect::<Vec<_>>();

    assert_eq!(conditions.len(), 2);
    assert_ne!(conditions[0].branch_id, conditions[1].branch_id);
    assert!(
        conditions
            .iter()
            .all(|branch| { catalog.unclassified_branch_ids.contains(&branch.branch_id) })
    );
    Ok(())
}

#[test]
fn branch_catalog_source_errors_identify_the_edge_and_span() -> TestResult {
    let (mut state, _discovered, root) = scan_source_fixture(
        "branch-catalog-source-error-context",
        &[("src/lib.rs", "fn run(value: bool) { if value {} }")],
        "src/lib.rs",
    )?;
    let edge = state
        .edges
        .iter_mut()
        .find(|edge| edge.kind == "if")
        .ok_or_else(|| test_error("if branch edge missing from source-error fixture"))?;
    edge.span = "katana:src/lib.rs:1:999-1:999".to_string();

    let error = crate::source_closure::branch_catalog::materialize_branch_catalog(
        test_manifest_root(),
        &state,
        &test_profiles(),
        &root,
    )
    .err()
    .ok_or_else(|| test_error("invalid branch span unexpectedly materialized"))?;

    for expected in [
        "branch catalog source extraction failed",
        "file=src/lib.rs",
        "edge_id=edge:",
        "kind=if",
        "span=katana:src/lib.rs:1:999-1:999",
    ] {
        assert!(error.contains(expected), "missing {expected}: {error}");
    }
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn branch_catalog_bytes_are_stable_for_identical_source_facts() -> TestResult {
    let first = catalog_bytes_for_source(
        "branch-stability-a",
        "fn run(value: bool) { if value { } match value { true => {}, false => {} } }",
    )?;
    let second = catalog_bytes_for_source(
        "branch-stability-b",
        "fn run(value: bool) { if value { } match value { true => {}, false => {} } }",
    )?;
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn branch_catalog_id_and_artifact_change_when_branch_syntax_changes() -> TestResult {
    let first = catalog_for_source(
        "branch-syntax-a",
        "fn run(value: bool) { if value { } match value { true => {}, false => {} } }",
    )?;
    let second = catalog_for_source(
        "branch-syntax-b",
        "fn run(value: bool) { if !value { } match value { true => {}, false => {} } }",
    )?;
    let first_if = first
        .branches
        .iter()
        .find(|branch| branch.kind == "if")
        .ok_or_else(|| test_error("if branch missing from first catalog"))?;
    let second_if = second
        .branches
        .iter()
        .find(|branch| branch.kind == "if")
        .ok_or_else(|| test_error("if branch missing from second catalog"))?;

    assert_ne!(first_if.branch_id, second_if.branch_id);
    assert_ne!(
        serde_json::to_vec_pretty(&first)?,
        serde_json::to_vec_pretty(&second)?
    );
    Ok(())
}

#[test]
fn branch_catalog_id_and_artifact_change_when_source_span_changes() -> TestResult {
    let first = catalog_for_source(
        "branch-span-a",
        "fn run(value: bool) { if value { } match value { true => {}, false => {} } }",
    )?;
    let second = catalog_for_source(
        "branch-span-b",
        "\nfn run(value: bool) { if value { } match value { true => {}, false => {} } }",
    )?;
    let first_if = first
        .branches
        .iter()
        .find(|branch| branch.kind == "if")
        .ok_or_else(|| test_error("if branch missing from first catalog"))?;
    let second_if = second
        .branches
        .iter()
        .find(|branch| branch.kind == "if")
        .ok_or_else(|| test_error("if branch missing from second catalog"))?;

    assert_ne!(first_if.branch_id, second_if.branch_id);
    assert_ne!(
        serde_json::to_vec_pretty(&first)?,
        serde_json::to_vec_pretty(&second)?
    );
    Ok(())
}
