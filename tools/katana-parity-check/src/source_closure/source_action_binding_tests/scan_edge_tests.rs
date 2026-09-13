use super::super::super::edge_model::{ClosureEdge, LexicalResolutionStatus};
use super::super::SourceActionBindingReport;
use super::fixture::actual_fixture;

#[test]
fn unresolved_scan_edge_kind_is_structured_and_fingerprinted()
-> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut state = facts.state;
    state.unresolved_edges.push(ClosureEdge {
        id: "edge:fixture".into(),
        from_file: "crates/katana-ui/src/editor.rs".into(),
        kind: "call".into(),
        from_symbol: "emit".into(),
        to_path: None,
        to_symbol: Some("render".into()),
        lexical_resolution: Some(LexicalResolutionStatus::Unresolved),
        span: "katana:crates/katana-ui/src/editor.rs:1:1-1:4".into(),
    });
    let first = SourceActionBindingReport::build(&state, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    let entry = first
        .unresolved
        .iter()
        .find(|entry| {
            entry.kind == "unresolved_scan_edge"
                && entry.span.as_deref() == Some("katana:crates/katana-ui/src/editor.rs:1:1-1:4")
        })
        .ok_or("unresolved scan edge missing")?;
    assert_eq!(entry.source_edge_kind.as_deref(), Some("call"));
    assert_eq!(entry.source_edge_detail.as_deref(), Some("render"));
    assert_eq!(
        entry.source_edge_resolution,
        Some(LexicalResolutionStatus::Unresolved)
    );
    let mut changed = state;
    changed.unresolved_edges[0].lexical_resolution = Some(LexicalResolutionStatus::Ambiguous);
    let second = SourceActionBindingReport::build(&changed, &facts.source, &facts.requirements)
        .map_err(std::io::Error::other)?;
    assert_ne!(first.fingerprint, second.fingerprint);
    Ok(())
}

#[test]
fn unknown_lexical_resolution_status_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let edge = r#"{"id":"edge:fixture","from_file":"fixture.rs","kind":"call","from_symbol":"emit","to_path":null,"to_symbol":null,"lexical_resolution":"unknown","span":"fixture.rs:1"}"#;
    assert!(serde_json::from_str::<ClosureEdge>(edge).is_err());
    Ok(())
}

#[test]
fn whitespace_unresolved_scan_edge_detail_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut state = facts.state;
    state.unresolved_edges.push(ClosureEdge {
        id: "edge:fixture".into(),
        from_file: "crates/katana-ui/src/editor.rs".into(),
        kind: "call".into(),
        from_symbol: "emit".into(),
        to_path: None,
        to_symbol: Some(" \t".into()),
        lexical_resolution: None,
        span: "katana:crates/katana-ui/src/editor.rs:1:1-1:4".into(),
    });
    assert!(matches!(
        SourceActionBindingReport::build(&state, &facts.source, &facts.requirements),
        Err(error) if error == "unresolved scan edge detail is empty"
    ));
    Ok(())
}

#[test]
fn empty_unresolved_scan_edge_kind_fails_closed() -> Result<(), Box<dyn std::error::Error>> {
    let facts = actual_fixture()?;
    let mut state = facts.state;
    state.unresolved_edges.push(ClosureEdge {
        id: "edge:fixture".into(),
        from_file: "crates/katana-ui/src/editor.rs".into(),
        kind: " ".into(),
        from_symbol: "emit".into(),
        to_path: None,
        to_symbol: None,
        lexical_resolution: None,
        span: "katana:crates/katana-ui/src/editor.rs:1:1-1:4".into(),
    });
    assert!(matches!(
        SourceActionBindingReport::build(&state, &facts.source, &facts.requirements),
        Err(error) if error == "unresolved scan edge kind is empty"
    ));
    Ok(())
}
