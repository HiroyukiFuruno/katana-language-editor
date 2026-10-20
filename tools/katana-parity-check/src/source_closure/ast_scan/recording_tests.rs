use proc_macro2::Span;

use super::super::super::scan_state::ScanState;
use super::super::SourceClosureVisitor;

#[test]
fn source_scan_keeps_structural_edges_and_reports_only_unknown_call_unresolved()
-> Result<(), String> {
    let source = "fn run(enabled: bool) { if enabled { return; } missing(); }";
    let file = syn::parse_file(source).map_err(|error| error.to_string())?;
    let root = std::env::temp_dir().join("kle-recording-tests");
    let path = root.join("src/lib.rs");
    let mut state = ScanState::default();
    let mut discovered = Vec::new();

    SourceClosureVisitor::new(&root, &path, "src/lib.rs", &mut state, &mut discovered)
        .scan_file(&file);

    for kind in ["if", "if_then", "return"] {
        assert!(
            state.edges.iter().any(|edge| edge.kind == kind),
            "missing structural edge {kind}"
        );
        assert!(!state.unresolved_edges.iter().any(|edge| edge.kind == kind));
    }
    assert!(state.unresolved_edges.iter().any(|edge| {
        edge.kind == "call"
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|symbol| symbol.contains("missing"))
    }));
    Ok(())
}

#[test]
fn unknown_record_branch_kind_is_an_explicit_unresolved_edge() {
    let root = std::env::temp_dir().join("kle-recording-tests-unknown-kind");
    let path = root.join("src/lib.rs");
    let mut state = ScanState::default();
    let mut discovered = Vec::new();
    let mut visitor =
        SourceClosureVisitor::new(&root, &path, "src/lib.rs", &mut state, &mut discovered);

    visitor.record_branch_edge("unknown_control_flow", Some("label"), Span::call_site());

    assert!(state.edges.iter().any(|edge| {
        edge.kind == "unknown_control_flow"
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|reason| reason.contains("unknown control-flow kind"))
    }));
    assert!(state.unresolved_edges.iter().any(|edge| {
        edge.kind == "unknown_control_flow"
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|reason| reason.contains("unknown control-flow kind"))
    }));
}
