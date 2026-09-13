use super::*;

#[test]
fn action_construction_index_records_definition_call_and_struct_spans() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "action-construction-forms",
        &[(
            "src/lib.rs",
            "enum AppAction { Bold(String), Link { url: String } }\nfn call() { let _ = AppAction::Bold(\"x\".to_string()); }\nfn structured() { let _ = AppAction::Link { url: \"u\".to_string() }; }",
        )],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    assert_eq!(artifact.actions.len(), 2);
    let bold = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Bold")
        .ok_or("bold action")?;
    assert!(
        bold.definition_span
            .contains("enum_span=katana:src/lib.rs:")
    );
    assert!(
        bold.definition_span
            .contains("variant_span=katana:src/lib.rs:")
    );
    assert_eq!(bold.construction_sites.len(), 1);
    assert!(bold.construction_sites[0].contains("symbol=call"));
    assert!(bold.construction_sites[0].contains("style=call"));
    let link = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Link")
        .ok_or("link action")?;
    assert_eq!(link.construction_sites.len(), 1);
    assert!(link.construction_sites[0].contains("symbol=structured"));
    assert!(link.construction_sites[0].contains("style=struct"));
    assert!(
        artifact
            .unresolved_action_origins
            .iter()
            .all(|entry| entry.contains("unproven"))
    );
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn action_construction_index_is_deterministic_and_changes_with_variant_or_span() -> TestResult {
    let first = action_origins_bytes_for_source(
        "action-deterministic-first",
        "enum AppAction { Bold(String) } fn run() { let _ = AppAction::Bold(String::new()); }",
    )?;
    let identical = action_origins_bytes_for_source(
        "action-deterministic-identical",
        "enum AppAction { Bold(String) } fn run() { let _ = AppAction::Bold(String::new()); }",
    )?;
    let changed_variant = action_origins_bytes_for_source(
        "action-deterministic-variant",
        "enum AppAction { Italic(String) } fn run() { let _ = AppAction::Italic(String::new()); }",
    )?;
    let changed_span = action_origins_bytes_for_source(
        "action-deterministic-span",
        "\nenum AppAction { Bold(String) } fn run() { let _ = AppAction::Bold(String::new()); }",
    )?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_variant);
    assert_ne!(first, changed_span);
    Ok(())
}

#[test]
fn action_construction_alias_and_ambiguous_definition_are_explicit_unresolved() -> TestResult {
    let (alias_state, _discovered, alias_root) = scan_source_fixture(
        "action-alias-unresolved",
        &[(
            "src/lib.rs",
            "enum AppAction { Bold(String) } use crate::AppAction as Alias; fn run() { let _ = Alias::Bold(String::new()); }",
        )],
        "src/lib.rs",
    )?;
    let alias_artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &alias_state,
    );
    assert!(
        alias_artifact
            .unresolved_action_origins
            .iter()
            .any(|entry| {
                entry.contains("imported AppAction-like construction path")
                    && entry.contains("style=call")
            })
    );
    cleanup_dir(alias_root)?;

    let (ambiguous_state, _discovered, ambiguous_root) = scan_source_fixture(
        "action-ambiguous-definition",
        &[(
            "src/lib.rs",
            "enum AppAction { Bold(String) } mod nested { enum AppAction { Bold(String) } } fn run() { let _ = AppAction::Bold(String::new()); }",
        )],
        "src/lib.rs",
    )?;
    let ambiguous_artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &ambiguous_state,
    );
    assert!(ambiguous_artifact.actions.is_empty());
    assert!(
        ambiguous_artifact
            .unresolved_action_origins
            .iter()
            .any(|entry| entry.contains("ambiguous indexed AppAction variant definitions"))
    );
    cleanup_dir(ambiguous_root)?;
    Ok(())
}

#[test]
fn action_construction_unknown_variant_and_macro_are_explicit_unresolved() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "action-unknown-and-macro",
        &[(
            "src/lib.rs",
            "enum AppAction { Bold(String) } fn run() { let _ = AppAction::Missing(String::new()); AppAction::Bold!(); }",
        )],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    assert!(artifact.unresolved_action_origins.iter().any(|entry| {
        entry.contains("no indexed AppAction variant definition") && entry.contains("Missing")
    }));
    assert!(artifact.unresolved_action_origins.iter().any(|entry| {
        entry.contains("macro construction cannot be proven") && entry.contains("style=macro")
    }));
    cleanup_dir(root)?;
    Ok(())
}
