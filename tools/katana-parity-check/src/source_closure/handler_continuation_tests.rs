use super::*;

#[test]
fn same_implementation_continuation_routes_are_provisional_and_fail_closed() -> TestResult {
    fn route_bytes(
        name: &str,
        source: &str,
    ) -> TestResult<(
        Vec<u8>,
        String,
        crate::source_closure::model::ActionOriginsArtifact,
    )> {
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        let route = artifact.actions[0].forward_route.join("|");
        let bytes = serde_json::to_vec(&artifact)?;
        cleanup_dir(root)?;
        Ok((bytes, route, artifact))
    }

    let unique_source = r#"
        enum AppAction { Bold }
        struct Editor;
        impl Editor {
            fn dispatch_action(&self, action: AppAction) {
                match action { AppAction::Bold => self.handle_bold(), _ => {} }
            }
            fn handle_bold(&self) { self.finish(); }
            fn finish(&self) { return; }
        }
    "#;
    let (first, route, artifact) = route_bytes("continuation-unique", unique_source)?;
    assert!(route.contains("same_implementation_continuation_route;status=provisional"));
    for expected in [
        "call_syntax=self.finish",
        "call_span=katana:src/lib.rs:",
        "source_definition_span=katana:src/lib.rs:",
        "target_definition_span=katana:src/lib.rs:",
        "implementation=inherent:Editor",
    ] {
        assert!(route.contains(expected), "missing {expected}: {route}");
    }
    assert!(
        artifact
            .actions
            .iter()
            .all(|action| action.origin_classification == "unresolved")
    );
    assert!(
        artifact
            .actions
            .iter()
            .all(|action| action.kle_leafs.is_empty())
    );

    let identical = route_bytes("continuation-unique-identical", unique_source)?.0;
    let changed_syntax = route_bytes(
        "continuation-syntax-change",
        &unique_source.replace("self.finish()", "self.complete()"),
    )?
    .0;
    let changed_span = route_bytes("continuation-span-change", &format!("\n{unique_source}"))?.0;
    assert_eq!(first, identical);
    assert_ne!(first, changed_syntax);
    assert_ne!(first, changed_span);

    for (name, body, expected) in [
        (
            "continuation-duplicate",
            "fn handle_bold(&self) { self.finish(); } fn finish(&self) { return; } fn finish(&mut self) { return; }",
            "multiple receiver-compatible inherent candidates",
        ),
        (
            "continuation-cross-impl",
            "fn handle_bold(&self) { self.finish(); }",
            "zero receiver-compatible inherent candidates",
        ),
        (
            "continuation-branch",
            "fn handle_bold(&self) { if true { self.finish(); } } fn finish(&self) { return; }",
            "branch-contained continuation",
        ),
        (
            "continuation-closure",
            "fn handle_bold(&self) { let callback = || self.finish(); } fn finish(&self) { return; }",
            "closure boundary",
        ),
        (
            "continuation-macro",
            "fn handle_bold(&self) { println!(\"boundary\"); self.finish(); } fn finish(&self) { return; }",
            "macro boundary",
        ),
        (
            "continuation-direct-recursion",
            "fn handle_bold(&self) { self.handle_bold(); }",
            "direct recursion",
        ),
        (
            "continuation-mutual-recursion",
            "fn handle_bold(&self) { self.next(); } fn next(&self) { self.handle_bold(); }",
            "mutually recursive continuation loop",
        ),
        (
            "continuation-no-terminal",
            "fn handle_bold(&self) { self.finish(); } fn finish(&self) { let value = 1; let _ = value; }",
            "no terminal candidate",
        ),
    ] {
        let source = if name == "continuation-cross-impl" {
            format!(
                "enum AppAction {{ Bold }} struct Editor; struct Other; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} {body} }} impl Other {{ fn finish(&self) {{ return; }} }}"
            )
        } else {
            format!(
                "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} {body} }}"
            )
        };
        let (_bytes, route, artifact) = route_bytes(name, &source)?;
        assert!(route.contains(expected), "missing {expected}: {route}");
        assert!(
            artifact
                .actions
                .iter()
                .all(|action| action.origin_classification == "unresolved")
        );
    }
    Ok(())
}

#[test]
fn handler_body_index_records_spans_syntax_and_all_boundary_facts() -> TestResult {
    let source = r#"
        enum AppAction { Bold }
        struct Editor;
        impl Editor {
            fn dispatch_action(&self, action: AppAction) {
                match action { AppAction::Bold => self.handle_bold(), _ => {} }
            }
            #[cfg(target_os = "macos")]
            fn handle_bold(&self) {
                let mut value = 0;
                value = self.read_value();
                if value > 0 { self.inner_call(); }
                match value { 0 => self.inner_call(), _ => self.inner_call() }
                let _closure = || self.inner_call();
                other.inner_call();
                external::native_call();
                println!("boundary");
                return value;
                return;
            }
        }
    "#;
    let (state, _discovered, root) = scan_source_fixture(
        "handler-body-facts",
        &[("src/lib.rs", source)],
        "src/lib.rs",
    )?;
    let artifact = crate::source_closure::action_origins::materialize_action_origins(
        test_manifest_root(),
        &state,
    );
    let route = &artifact.actions[0].forward_route[0];
    for expected in [
        "handler_body_fact=kind:cfg",
        "handler_body_fact=kind:direct_method_call;syntax:self.read_value",
        "handler_body_fact=kind:assignment",
        "handler_body_fact=kind:if_branch",
        "handler_body_fact=kind:match_branch",
        "handler_body_fact=kind:closure_boundary",
        "handler_body_fact=kind:macro_boundary;syntax:println",
        "handler_body_fact=kind:return;syntax:early_return",
        "unresolved import or external-native path call boundary",
        "trait or dynamic receiver call boundary",
        "closure boundary",
        "macro boundary",
        "multiple terminal-looking return facts",
        "handler body final effect remains unclassified",
    ] {
        assert!(route.contains(expected), "missing {expected}: {route}");
    }
    assert!(route.contains("span=katana:src/lib.rs:"), "{route}");
    assert!(
        artifact
            .unresolved_action_origins
            .iter()
            .any(|entry| { entry.contains("AppAction::Bold") && entry.contains("unproven") })
    );
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn handler_body_facts_are_deterministic_and_change_with_body_span_or_syntax() -> TestResult {
    fn bytes(name: &str, body: &str) -> TestResult<Vec<u8>> {
        let source = format!(
            "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} fn handle_bold(&self) {{ {body} }} }}"
        );
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", &source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        let bytes = serde_json::to_vec(&artifact)?;
        cleanup_dir(root)?;
        Ok(bytes)
    }

    let first = bytes("handler-body-stable-a", "self.read_value();")?;
    let identical = bytes("handler-body-stable-b", "self.read_value();")?;
    let changed_syntax = bytes("handler-body-changed-syntax", "self.write_value();")?;
    let changed_span = bytes("handler-body-changed-span", "\nself.read_value();")?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_syntax);
    assert_ne!(first, changed_span);
    Ok(())
}
