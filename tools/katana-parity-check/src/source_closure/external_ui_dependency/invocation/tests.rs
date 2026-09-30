use super::super::scan_katana_sources;

fn scan(source: &str) -> (usize, Vec<String>) {
    let (edges, unresolved) = scan_katana_sources(&[("src/editor.rs".into(), source.into())]);
    let shortcuts = edges
        .iter()
        .filter(|edge| edge.dependency_symbol == "InputState::consume_shortcut")
        .count();
    (shortcuts, unresolved)
}

fn event_edges(source: &str) -> Vec<(String, String)> {
    let (edges, _) = scan_katana_sources(&[("src/editor.rs".into(), source.into())]);
    edges
        .into_iter()
        .filter(|edge| edge.dependency_symbol == "Event::Paste")
        .map(|edge| (edge.kind, edge.span))
        .collect()
}

#[test]
fn actual_fixed_source_shape_records_input_state_shortcut() {
    let (shortcuts, unresolved) = scan(
        r#"
            use eframe::egui;
            fn command_shortcut_consumed(ctx: &egui::Context, raw_shortcut: &str) -> bool {
                let Some(parsed) = ShortcutKeyOps::parse_shortcut(raw_shortcut) else {
                    return false;
                };
                ctx.input_mut(|i| i.consume_shortcut(&parsed))
            }
        "#,
    );
    assert_eq!(shortcuts, 1);
    assert!(
        unresolved
            .iter()
            .all(|reason| !reason.contains("InputState::consume_shortcut"))
    );
}

#[test]
fn shortcut_closure_keeps_independent_event_seed() {
    let (edges, unresolved) = scan_katana_sources(&[(
        "src/editor.rs".into(),
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { let _ = Event::Paste; i.consume_shortcut(&parsed) }); }".into(),
    )]);
    assert!(
        edges
            .iter()
            .any(|edge| edge.dependency_symbol == "InputState::consume_shortcut")
    );
    assert!(
        edges
            .iter()
            .any(|edge| edge.dependency_symbol == "Event::Paste")
    );
    assert!(
        unresolved
            .iter()
            .all(|reason| !reason.contains("InputState::consume_shortcut"))
    );
}

#[test]
fn actual_fixed_paste_pattern_records_exact_pattern_span() {
    let edges = event_edges(
        "fn run() {\n    match event {\n        egui::Event::Paste(text) => consume(text),\n        _ => {}\n    }\n}",
    );
    assert_eq!(
        edges,
        vec![(
            "event-pattern".into(),
            "katana:src/editor.rs:3:8-3:32".into()
        )]
    );
}

#[test]
fn event_pattern_keeps_unqualified_candidate_and_rejects_other_variants() {
    let edges = event_edges(
        "fn run() {\n    match event {\n        Event::Paste(text) if text.len() > 0 => consume(text),\n        OtherEvent::Paste(text) => consume(text),\n        _ => {}\n    }\n}",
    );
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].0, "event-pattern");
}

#[test]
fn event_pattern_ignores_strings_comments_and_macros() {
    let edges = event_edges(
        r#"
            fn run() {
                let _ = "egui::Event::Paste(text)";
                /* WHY: egui::Event::Paste(text) is not a syntax node here. */
                emit!(egui::Event::Paste(text));
            }
        "#,
    );
    assert!(edges.is_empty());
}

#[test]
fn rejects_shadowed_other_receiver_and_escaped_closure() {
    for source in [
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { let i = other; i.consume_shortcut(&parsed) }); }",
        "fn run(other: &Other) { other.input_mut(|i| i.consume_shortcut(&parsed)); }",
        "fn run(ctx: &egui::Context) { let callback = |i| i.consume_shortcut(&parsed); ctx.input_mut(callback); }",
    ] {
        let (shortcuts, unresolved) = scan(source);
        assert_eq!(shortcuts, 0, "{source}");
        assert!(!unresolved.is_empty(), "{source}");
    }
}

#[test]
fn rejects_pattern_shadowing_and_local_item_body_calls() {
    for source in [
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { let i: Other = other; i.consume_shortcut(&parsed) }); }",
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { let (i,) = (other,); i.consume_shortcut(&parsed) }); }",
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { for i in inputs { i.consume_shortcut(&parsed); } }); }",
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { match value { i => i.consume_shortcut(&parsed) } }); }",
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { if let i = other { i.consume_shortcut(&parsed); } }); }",
        "fn run(ctx: &egui::Context) { ctx.input_mut(|i| { fn fake(i: Other) { i.consume_shortcut(&parsed); } }); }",
    ] {
        let (shortcuts, unresolved) = scan(source);
        assert_eq!(shortcuts, 0, "{source}");
        assert!(!unresolved.is_empty(), "{source}");
    }
}

#[test]
fn rejects_outer_context_rebinding_and_local_context_impl() {
    for source in [
        "fn run(ctx: &egui::Context) { let ctx: Other = other; ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
        "fn run(ctx: &egui::Context) { let ctx = other; ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
        "struct Context; impl Context { fn run(&self) { self.input_mut(|i| i.consume_shortcut(&parsed)); } }",
    ] {
        let (shortcuts, unresolved) = scan(source);
        assert_eq!(shortcuts, 0, "{source}");
        assert!(!unresolved.is_empty(), "{source}");
    }
}

#[test]
fn nested_context_import_does_not_type_outer_binding() {
    let (shortcuts, unresolved) = scan(
        "mod nested { use egui::Context; fn inner(ctx: &Context) { ctx.input_mut(|i| i.consume_shortcut(&parsed)); } } fn outer(ctx: &Context) { ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
    );
    assert_eq!(shortcuts, 1);
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("input_mut receiver is not egui::Context"))
    );
}

#[test]
fn nested_module_local_context_type_does_not_use_parent_import() {
    let (shortcuts, unresolved) = scan(
        "use egui::Context; mod nested { struct Context; fn run(ctx: &Context) { ctx.input_mut(|i| i.consume_shortcut(&parsed)); } }",
    );
    assert_eq!(shortcuts, 0);
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("input_mut receiver is not egui::Context"))
    );
}

#[test]
fn local_context_types_exclude_same_scope_import_alias() {
    for source in [
        "use egui::Context; struct Context; fn run(ctx: &Context) { ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
        "use egui::Context; type Context = Other; fn run(ctx: &Context) { ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
    ] {
        let (shortcuts, unresolved) = scan(source);
        assert_eq!(shortcuts, 0, "{source}");
        assert!(
            unresolved
                .iter()
                .any(|reason| reason.contains("input_mut receiver is not egui::Context")),
            "{source}"
        );
    }
}
