use super::*;

#[test]
fn branch_outcome_candidates_are_individual_ast_facts_and_fail_closed() -> TestResult {
    fn route_bytes(
        name: &str,
        body: &str,
    ) -> TestResult<(
        Vec<u8>,
        String,
        crate::source_closure::scan_state::ScanState,
    )> {
        let source = format!(
            "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} fn handle_bold(&self, value: bool) {{ {body} }} }}"
        );
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", &source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        let route = artifact.actions[0].forward_route.join("|");
        let bytes = serde_json::to_vec(&artifact)?;
        assert!(
            artifact
                .actions
                .iter()
                .all(|action| action.origin_classification == "unresolved")
        );
        cleanup_dir(root)?;
        Ok((bytes, route, state))
    }

    let body = r#"
        if value { } else { return; }
        if value { self.inner(); }
        match value {
            true if value => {},
            false => return,
            _ => { self.inner(); }
        }
        if value { match value { true => return, _ => {} } }
        let callback = || self.callback();
        dynamic_target()(value);
        other.inner();
        println!("boundary");
    "#;
    let (first, route, state) = route_bytes("branch-outcomes", body)?;
    let definition = state
        .method_definitions
        .iter()
        .find(|definition| definition.method == "handle_bold")
        .ok_or_else(|| test_error("handler definition missing"))?;
    let outcomes = &definition.branch_outcome_candidates;
    assert_eq!(
        outcomes
            .iter()
            .filter(|candidate| candidate.kind == "if_then")
            .count(),
        3
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|candidate| candidate.kind == "if_else")
            .count(),
        1
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|candidate| candidate.kind == "if_absent_else")
            .count(),
        2
    );
    assert_eq!(
        outcomes
            .iter()
            .filter(|candidate| candidate.kind == "match_arm")
            .count(),
        5
    );
    assert!(
        outcomes
            .iter()
            .any(|candidate| candidate.body_shape == "empty")
    );
    assert!(
        outcomes
            .iter()
            .any(|candidate| candidate.body_shape == "early_return")
    );
    assert!(
        outcomes
            .iter()
            .any(|candidate| candidate.body_shape == "fallthrough")
    );
    assert!(outcomes.iter().all(|candidate| {
        candidate.span.starts_with("katana:src/lib.rs:")
            && candidate.body_span.starts_with("katana:src/lib.rs:")
            && candidate
                .unresolved
                .iter()
                .any(|reason| reason == "branch outcome semantic effect remains unresolved")
    }));
    assert!(outcomes.iter().any(|candidate| {
        candidate.syntax.contains("true if value =>")
            && candidate
                .unresolved
                .iter()
                .any(|reason| reason.contains("semantic effect"))
    }));
    assert!(outcomes.iter().any(|candidate| {
        !candidate.enclosing_outcome_syntax.is_empty()
            && candidate
                .terminal_effect_candidates
                .iter()
                .any(|terminal| terminal.kind == "return")
    }));
    for expected in [
        "closure boundary",
        "dynamic call boundary",
        "trait or dynamic receiver call boundary",
        "macro boundary",
        "branch_outcome_candidate=kind:if_then",
        "branch_outcome_candidate=kind:if_else",
        "branch_outcome_candidate=kind:if_absent_else",
        "branch_outcome_candidate=kind:match_arm",
        "resolution=unresolved",
    ] {
        assert!(route.contains(expected), "missing {expected}: {route}");
    }

    let (identical, _, _) = route_bytes("branch-outcomes-identical", body)?;
    let (changed_syntax, _, _) = route_bytes(
        "branch-outcomes-syntax-change",
        &body.replace("callback", "changed_callback"),
    )?;
    let (changed_span, _, _) = route_bytes("branch-outcomes-span-change", &format!("\n{body}"))?;
    assert_eq!(first, identical);
    assert_ne!(first, changed_syntax);
    assert_ne!(first, changed_span);
    Ok(())
}

#[test]
fn terminal_effect_candidate_index_is_ast_provenance_only_and_fails_closed() -> TestResult {
    fn bytes(name: &str, body: &str) -> TestResult<(Vec<u8>, String, bool)> {
        let source = format!(
            "enum AppAction {{ Bold }} struct Editor; impl Editor {{ fn dispatch_action(&self, action: AppAction) {{ match action {{ AppAction::Bold => self.handle_bold(), _ => {{}} }} }} fn handle_bold(&self) {{ {body} }} }}"
        );
        let (state, _discovered, root) =
            scan_source_fixture(name, &[("src/lib.rs", &source)], "src/lib.rs")?;
        let artifact = crate::source_closure::action_origins::materialize_action_origins(
            test_manifest_root(),
            &state,
        );
        let route = artifact.actions[0].forward_route[0].clone();
        let bytes = serde_json::to_vec(&artifact)?;
        cleanup_dir(root)?;
        let unresolved = artifact
            .unresolved_action_origins
            .iter()
            .any(|entry| entry.contains("action origin remains unproven"));
        Ok((bytes, route, unresolved))
    }

    let body = r#"
        self.value = 1;
        if self.value > 0 {
            self.continue_edit();
        }
        match self.value { 0 => self.other_step(), _ => return }
        external::native_call();
        other.inner_call();
        let callback = || self.callback_step();
        dynamic_target()(self.value);
        println!("macro boundary");
    "#;
    let (first, route, unresolved) = bytes("terminal-candidates-a", body)?;
    let (identical, identical_route, _identical_unresolved) = bytes("terminal-candidates-b", body)?;
    assert_eq!(first, identical);
    assert_eq!(route, identical_route);
    for expected in [
        "terminal_effect_candidate=kind:direct_self_receiver_assignment",
        "terminal_effect_candidate=kind:same_implementation_continuation_call;syntax:self.continue_edit",
        "terminal_effect_candidate=kind:external_native_boundary;syntax:external::native_call",
        "terminal_effect_candidate=kind:return;syntax:return",
        "terminal_effect_candidate=kind:closure_boundary",
        "terminal_effect_candidate=kind:dynamic_boundary",
        "terminal_effect_candidate=kind:macro_boundary;syntax:println",
        "enclosing=if_branch:self.value > 0",
        "enclosing=match_branch:self.value",
        "trait or dynamic receiver call boundary",
        "unresolved import or external-native path call boundary",
        "multiple terminal-effect candidate facts",
    ] {
        assert!(route.contains(expected), "missing {expected}: {route}");
    }
    assert!(unresolved);

    let (changed_syntax, _, _) = bytes(
        "terminal-candidates-syntax-change",
        &body.replace("continue_edit", "changed_edit"),
    )?;
    let (changed_span, _, _) = bytes("terminal-candidates-span-change", &format!("\n{body}"))?;
    assert_ne!(first, changed_syntax);
    assert_ne!(first, changed_span);

    let (_zero_bytes, zero_route, zero_unresolved) =
        bytes("terminal-candidates-zero", "let value = 1;")?;
    assert!(zero_route.contains("no terminal-effect candidate facts"));
    assert!(zero_route.contains("terminal_effect_candidate=none"));
    assert!(zero_unresolved);
    Ok(())
}
