use super::*;
use syn::visit::Visit;

fn lint(source: &str) -> Vec<Violation> {
    let Ok(file) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut visitor = Visitor::new("fixture.rs".into());
    visitor.visit_file(&file);
    visitor.violations
}

#[test]
fn rejects_action_contract_types_and_event_paths() {
    for source in [
        "enum EditorAction { SaveDocument }",
        "struct EditorActionRequest { action: EditorAction }",
        "enum EditorActionSource { Toolbar }",
        "trait EditorActionControl {}",
        "fn f(event: EditorEvent) { let _ = EditorEvent::ActionRequested(event); }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_host_payload_and_mapping_constructs() {
    for source in [
        "struct EditorScrollSyncRequest { logical_position: usize }",
        "fn map_save_action() {}",
        "fn dispatch_diagnostic_request() {}",
        "fn f() { let _ = EditorAction::SetViewMode(ViewMode::Split); }",
        "fn f(event: EditorEvent) { assert!(matches!(event, EditorEvent::ActionRequested(_))); }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn accepts_generic_receipts_and_source_manifests() {
    let source = r#"
        struct OpaqueEventReceipt { revision: u64, correlation: u64 }
        struct FeatureManifest { name: &'static str, label: &'static str }
        enum SourceMarker { WorkspaceMarkdownResultJump, DiagnosticPreview }
        fn forward_once(event: OpaqueEvent) -> OpaqueReceipt { consume(event) }
        fn render_diagnostic_action(action: DiagnosticAction) { let _ = action; }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn accepts_generic_action_word_without_kle_semantics() {
    let source = "struct ButtonActionReceipt { class: &'static str } fn consume_action(receipt: ButtonActionReceipt) { let _ = receipt; }";
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}
