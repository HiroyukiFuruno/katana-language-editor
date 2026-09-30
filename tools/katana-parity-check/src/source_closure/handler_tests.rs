use super::*;

#[test]
fn dispatch_arm_index_records_direct_forms_and_unresolved_routes() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "dispatch-arm-index",
        &[(
            "src/lib.rs",
            r#"
                enum AppAction { Bold(String), Link { url: String }, Other }
                fn dispatch_action(action: AppAction) {
                    match action {
                        AppAction::Bold(_) => self_handle_bold(),
                        AppAction::Link { .. } | AppAction::Bold(_) => if true { self_handle_link(); }
                        AppAction::Other => { self_handle_bold(); self_handle_link(); }
                        _ => {}
                    }
                }
                fn self_handle_bold() {}
                fn self_handle_link() {}
            "#,
        )],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let bold = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Bold")
        .ok_or_else(|| test_error("bold action missing"))?;
    assert_eq!(bold.forward_route.len(), 2);
    assert!(
        bold.forward_route
            .iter()
            .any(|route| route.contains("action=AppAction::Bold")
                && route.contains("handlers=self_handle_bold"))
    );
    assert!(bold.forward_route.iter().any(|route| {
        route.contains("variant_pattern_span=katana:src/lib.rs:")
            && route.contains("handlers=self_handle_link")
            && route.contains("nested condition")
    }));
    let link = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Link")
        .ok_or_else(|| test_error("link action missing"))?;
    assert_eq!(link.forward_route.len(), 1);
    assert!(link.forward_route[0].contains("handlers=self_handle_link"));
    let other = artifact
        .actions
        .iter()
        .find(|action| action.action_id == "AppAction::Other")
        .ok_or_else(|| test_error("other action missing"))?;
    assert!(other.forward_route[0].contains("multiple direct handler calls"));
    assert!(
        artifact
            .unresolved_action_origins
            .iter()
            .any(|entry| entry.contains("dispatch_fallthrough_unresolved")
                && entry.contains("fallthrough target"))
    );
    assert!(artifact.unresolved_action_origins.iter().all(
        |entry| entry.contains("unproven") || entry.contains("dispatch_fallthrough_unresolved")
    ));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn dispatch_route_bytes_are_deterministic_and_track_variant_and_span_mutations() -> TestResult {
    let source = "enum AppAction { Bold(String) } fn dispatch_action(action: AppAction) { match action { AppAction::Bold(_) => self.handle_bold(), _ => {} } }";
    let first = action_origins_bytes_for_source("dispatch-route-bytes-a", source)?;
    let identical = action_origins_bytes_for_source("dispatch-route-bytes-b", source)?;
    let changed_variant = action_origins_bytes_for_source(
        "dispatch-route-bytes-variant",
        "enum AppAction { Italic(String) } fn dispatch_action(action: AppAction) { match action { AppAction::Italic(_) => self.handle_italic(), _ => {} } }",
    )?;
    let changed_span = action_origins_bytes_for_source(
        "dispatch-route-bytes-span",
        "\nenum AppAction { Bold(String) } fn dispatch_action(action: AppAction) { match action { AppAction::Bold(_) => self.handle_bold(), _ => {} } }",
    )?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_variant);
    assert_ne!(first, changed_span);
    Ok(())
}

#[test]
fn handler_definition_index_links_only_one_compatible_inherent_method() -> TestResult {
    let source = r#"
        enum AppAction { Bold }
        struct Editor;
        impl Editor {
            fn dispatch_action(&self, action: AppAction) {
                match action { AppAction::Bold => self.handle_bold(), _ => {} }
            }
            fn handle_bold(&self) {}
        }
    "#;
    let (state, _discovered, root) =
        scan_source_fixture("handler-unique", &[("src/lib.rs", source)], "src/lib.rs")?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let route = &artifact.actions[0].forward_route[0];
    assert!(
        route.contains("handler_definition=file=src/lib.rs;implementation=inherent:Editor"),
        "{route}"
    );
    assert!(route.contains("implementation=inherent:Editor;method=handle_bold;receiver=&self"));
    assert!(route.contains("handler_call_fact=syntax:self.handle_bold;kind=self_method"));
    cleanup_dir(root)?;

    for (name, method_definition, expected) in [
        (
            "duplicate",
            "fn handle_bold(&self) {} fn handle_bold(&mut self) {}",
            "duplicate inherent",
        ),
        ("missing", "", "missing inherent"),
        (
            "trait-only",
            "trait Handles { fn handle_bold(&self); }",
            "trait-only",
        ),
        (
            "receiver",
            "fn handle_bold(self: Box<Editor>) {}",
            "receiver-incompatible",
        ),
        ("static", "fn handle_bold(value: u8) {}", "static inherent"),
    ] {
        let source = if method_definition.starts_with("trait") {
            "enum AppAction { Bold } struct Editor; trait Handles { fn handle_bold(&self); } impl Editor { fn dispatch_action(&self, action: AppAction) { match action { AppAction::Bold => self.handle_bold(), _ => {} } } }".to_string()
        } else {
            format!(
                "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} {method_definition} }}"
            )
        };
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", &source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        assert!(
            artifact.actions[0].forward_route[0].contains(expected),
            "{name}"
        );
        assert!(artifact.actions[0].forward_route[0].contains("handler_definition=unresolved"));
        cleanup_dir(root)?;
    }
    Ok(())
}

#[test]
fn handler_direct_path_call_records_one_local_function_as_provisional_only() -> TestResult {
    let source = r#"
        enum AppAction { Bold }
        fn dispatch_action(action: AppAction) {
            match action { AppAction::Bold => handle_bold(), _ => {} }
        }
        fn handle_bold() {}
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "handler-path-unique",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let action = &artifact.actions[0];
    let route = &action.forward_route[0];
    assert!(
        route.contains("handler_path_resolution=provisional"),
        "{route}"
    );
    assert!(
        route.contains("call=handle_bold;call_span=katana:src/lib.rs:"),
        "{route}"
    );
    assert!(route.contains("definition_file=src/lib.rs"), "{route}");
    assert!(
        route.contains("function_span=katana:src/lib.rs:"),
        "{route}"
    );
    assert_eq!(action.origin_classification, "unresolved");
    assert!(action.kle_leafs.is_empty());
    assert!(action.state_span.is_none());
    assert!(action.predecessor_actions.is_none());
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn handler_direct_path_call_rejects_alias_branch_and_recursive_routes() -> TestResult {
    for (name, source, expected) in [
        (
            "alias",
            "use crate::handle_bold as alias; enum AppAction { Bold } fn dispatch_action(action: AppAction) { match action { AppAction::Bold => alias(), _ => {} } } fn handle_bold() {}",
            "reason=unresolved:alias",
        ),
        (
            "branch",
            "enum AppAction { Bold } fn dispatch_action(action: AppAction) { match action { AppAction::Bold => if true { handle_bold() }, _ => {} } } fn handle_bold() {}",
            "reason=branch-or-multiple-call",
        ),
        (
            "recursive",
            "enum AppAction { Bold } fn dispatch_action(action: AppAction) { match action { AppAction::Bold => handle_bold(), _ => {} } } fn handle_bold() { handle_bold(); }",
            "reason=recursive-function-route",
        ),
    ] {
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        let route = &artifact.actions[0].forward_route[0];
        assert!(route.contains(expected), "{name}: {route}");
        assert_eq!(artifact.actions[0].origin_classification, "unresolved");
        assert!(artifact.actions[0].kle_leafs.is_empty());
        cleanup_dir(root)?;
    }
    Ok(())
}

#[test]
fn handler_definition_facts_are_deterministic_and_mutate_with_definition_changes() -> TestResult {
    fn bytes(name: &str, method: &str) -> TestResult<Vec<u8>> {
        let source = format!(
            "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} {method} }}"
        );
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", &source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        assert!(
            artifact
                .actions
                .iter()
                .all(|action| action.origin_classification == "unresolved")
        );
        assert!(
            artifact
                .unresolved_action_origins
                .iter()
                .any(|entry| { entry.contains("terminal_effect_candidate=") })
        );
        assert!(
            artifact
                .unresolved_action_origins
                .iter()
                .any(|entry| entry.contains("action origin remains unproven"))
        );
        let bytes = serde_json::to_vec(&artifact)?;
        cleanup_dir(root)?;
        Ok(bytes)
    }

    let first = bytes("handler-definition-stable-a", "fn handle_bold(&self) {}")?;
    let identical = bytes("handler-definition-stable-b", "fn handle_bold(&self) {}")?;
    let changed_method = bytes(
        "handler-definition-changed-method",
        "fn handle_bold(&mut self) {}",
    )?;
    let changed_span = bytes(
        "handler-definition-changed-span",
        "\nfn handle_bold(&self) {}",
    )?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_method);
    assert_ne!(first, changed_span);
    Ok(())
}
