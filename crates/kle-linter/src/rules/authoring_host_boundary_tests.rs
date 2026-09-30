use super::*;
use syn::visit::Visit;

fn lint(source: &str) -> Vec<Violation> {
    let Ok(file) = syn::parse_file(source) else {
        return Vec::new();
    };
    let mut visitor = Visitor::new("fixture.rs".into());
    visitor.visit_file(&file);
    visitor.into_violations()
}

#[test]
fn rejects_forbidden_core_types_and_methods() {
    for source in [
        "struct EditorAuthoringCommand;",
        "enum EditorCodeBlockKind { A }",
        "struct EditorAuthoringMenuState;",
        "struct EditorAuthoringControl;",
        "fn run_authoring_command() {}",
        "impl Editor { fn run_authoring_command(&self) {} }",
        "type Op = MarkdownAuthoringOp;",
        "type Kind = CodeBlockKind;",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_action_mapping_transform_and_popup_state() {
    for source in [
        "fn map_authoring_command() {}",
        "fn command_to_operation() {}",
        "fn map_code_block_kind_to_action() {}",
        "fn markdown_transform() {}",
        "fn f() { let _ = AppAction::AuthorMarkdown; }",
        "struct AuthoringPopupState { cursor: usize, selection_range: usize }",
        "fn authoring_popup() { let active_cursor = 1; }",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn accepts_static_visual_catalog_and_opaque_forwarding() {
    let source = r#"
        struct VisualLeaf { label: &'static str, icon: &'static str, order: usize, correlation: u64 }
        static AUTHORING_VISUAL_CATALOG: &[VisualLeaf] = &[];
        fn forward_opaque_event(event: OpaqueEvent) { relay_once(event); }
        fn consume_receipt(receipt: OpaqueReceipt) { dispatch_once(receipt); }
    "#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn accepts_generic_low_level_editor_type() {
    assert!(lint("struct TextEditor { cursor: usize, selection: Option<usize> }").is_empty());
}
