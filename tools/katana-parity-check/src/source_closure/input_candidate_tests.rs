use super::*;

#[test]
fn input_origin_candidate_index_records_primary_candidate_kinds_unclassified() -> TestResult {
    let source = r#"
        enum AppAction { Clicked, Shortcut, AccessKit, Callback, HostContinuation }
        fn ui(response: Response, ctx: Context, node: AccessNode) {
            if response.clicked() { let _ = AppAction::Clicked; }
            if ctx.consume_shortcut(&shortcut) { let _ = AppAction::Shortcut; }
            if node.accesskit_action_requested() { let _ = AppAction::AccessKit; }
            let _callback = || { let _ = AppAction::Callback; };
        }
        fn handle_pending_continuation() { let _ = AppAction::HostContinuation; }
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "input-origin-candidate-kinds",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );

    for (action_id, expected) in [
        (
            "AppAction::Clicked",
            "kind=egui_response_activation_condition;condition=response.clicked",
        ),
        (
            "AppAction::Shortcut",
            "kind=keyboard_shortcut_consumption_condition;condition=ctx.consume_shortcut",
        ),
        (
            "AppAction::AccessKit",
            "kind=accesskit_action_condition;condition=node.accesskit_action_requested",
        ),
        (
            "AppAction::Callback",
            "kind=callback_closure_boundary;condition=closure",
        ),
        (
            "AppAction::HostContinuation",
            "kind=non_ui_host_continuation;condition=non_ui_host_continuation_symbol",
        ),
    ] {
        let action = artifact
            .actions
            .iter()
            .find(|action| action.action_id == action_id)
            .ok_or("action")?;
        assert_eq!(action.origin_classification, "unresolved");
        assert!(
            action
                .forward_route
                .iter()
                .any(|route| route.contains("input_origin_candidate")
                    && route.contains(expected)
                    && route.contains("unresolved=")),
            "missing {expected} for {action_id}: {:?}",
            action.forward_route
        );
    }
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn input_origin_candidate_index_keeps_multiple_absent_macro_and_unknown_unresolved() -> TestResult {
    let source = r#"
        enum AppAction { Multiple, None, MacroUnknown, UnknownCondition }
        fn ui(response: Response, value: i32) {
            if response.clicked() {
                let _callback = || { let _ = AppAction::Multiple; };
            }
            if input_ready!() { let _ = AppAction::MacroUnknown; }
            if value > 0 { let _ = AppAction::UnknownCondition; }
        }
        fn plain() { let _ = AppAction::None; }
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "input-origin-candidate-unresolved",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );

    for expected in [
        "AppAction::Multiple",
        "multiple enclosing input candidates",
        "AppAction::None",
        "kind=absent_candidate",
        "AppAction::MacroUnknown",
        "macro-generated input condition cannot be proven",
        "AppAction::UnknownCondition",
        "if condition is not a structurally recognized input candidate",
    ] {
        assert!(
            artifact
                .unresolved_action_origins
                .iter()
                .any(|entry| entry.contains(expected)),
            "missing unresolved evidence {expected}: {:?}",
            artifact.unresolved_action_origins
        );
    }
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn unresolved_constructions_retain_input_origin_candidates() -> TestResult {
    let source = r#"
        enum AppAction { Known }
        use crate::AppAction as Alias;
        fn ui(response: Response) {
            if response.clicked() {
                let _callback = || {
                    let _ = Alias::Known;
                    let _ = AppAction::Missing;
                };
            }
        }
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "input-origin-candidate-unresolved-constructions",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );

    for expected in [
        "input_origin_candidate_index_unresolved",
        "action=Alias::Known",
        "imported AppAction-like construction path",
        "action=AppAction::Missing",
        "direct construction has no indexed AppAction variant definition",
        "kind=egui_response_activation_condition;condition=response.clicked",
        "kind=callback_closure_boundary;condition=closure",
        "multiple enclosing input candidates",
        "action origin remains unproven",
    ] {
        assert!(
            artifact
                .unresolved_action_origins
                .iter()
                .any(|entry| entry.contains(expected)),
            "missing unresolved construction candidate evidence {expected}: {:?}",
            artifact.unresolved_action_origins
        );
    }
    assert!(
        artifact
            .actions
            .iter()
            .all(|action| action.origin_classification == "unresolved")
    );
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn input_origin_candidate_bytes_are_deterministic_and_track_span_and_syntax_changes() -> TestResult
{
    let first = action_origins_bytes_for_source(
        "input-candidate-stable-a",
        "enum AppAction { Clicked } fn ui(response: Response) { if response.clicked() { let _ = AppAction::Clicked; } }",
    )?;
    let identical = action_origins_bytes_for_source(
        "input-candidate-stable-b",
        "enum AppAction { Clicked } fn ui(response: Response) { if response.clicked() { let _ = AppAction::Clicked; } }",
    )?;
    let changed_syntax = action_origins_bytes_for_source(
        "input-candidate-syntax",
        "enum AppAction { Clicked } fn ui(response: Response) { if response.secondary_clicked() { let _ = AppAction::Clicked; } }",
    )?;
    let changed_span = action_origins_bytes_for_source(
        "input-candidate-span",
        "\nenum AppAction { Clicked } fn ui(response: Response) { if response.clicked() { let _ = AppAction::Clicked; } }",
    )?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_syntax);
    assert_ne!(first, changed_span);
    Ok(())
}
