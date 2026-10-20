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
fn rejects_search_semantic_items_and_bindings() {
    for source in [
        "struct SearchState { active_index: usize, match_ranges: Vec<usize> }",
        "struct DocumentSearchQuery { query: String }",
        "enum DocumentSearchResult { Match { content: String } }",
        "fn f() { let regex_pattern = String::new(); let search_ranges = Vec::new(); }",
        "impl Editor { fn find_next(&mut self) {} }",
        "fn compute_matches() {}",
    ] {
        assert!(!lint(source).is_empty(), "accepted: {source}");
    }
}

#[test]
fn rejects_mutation_reached_from_search_code() {
    assert!(
        !lint("fn search_document(content: &mut String) { content.replace_range(.., \"x\"); }")
            .is_empty()
    );
    assert!(
        !lint("fn find_matches(content: &mut String) { replace_char_range(content); }").is_empty()
    );
}

#[test]
fn rejects_katana_search_action_mapping() {
    assert!(!lint("fn map() { let _ = EditorAction::Find; }").is_empty());
}

#[test]
fn accepts_kuc_opaque_types_display_scenarios_and_low_level_utility() {
    let source = r#"use katana_ui_core::molecule::structured::{ReplaceMode, SearchOptions}; struct SearchMotionSequence { stages: usize } enum FeatureGroup { DocumentFind, WorkspaceSearchResults } fn search_control_strings() -> String { "Search ⭐️".to_string() } fn replace_char_range(content: &mut String) { content.replace_range(.., "x"); }"#;
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}

#[test]
fn accepts_opaque_forwarding_without_semantic_processing() {
    let source = "fn forward_once(event: KucSearchEvent) { host.forward_events_once(event); }";
    assert!(
        lint(source).is_empty(),
        "unexpected violations: {:?}",
        lint(source)
    );
}
