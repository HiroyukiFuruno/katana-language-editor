use super::*;
use crate::source_closure::model::LexicalResolutionStatus;

#[test]
fn lexical_import_fixture_resolves_aliases_and_rejects_glob_edges() -> TestResult {
    let (state, discovered, root) = scan_source_fixture(
        "lexical-imports",
        &[
            (
                "src/lib.rs",
                "mod nested; mod target; use crate::target::call as renamed; use crate::{target as grouped_target, target::call as grouped_call}; use self::target::call as self_call; fn run() { renamed(); grouped_target::call(); grouped_call(); self_call(); }",
            ),
            (
                "src/nested.rs",
                "use super::target::call as parent_call; use super::target::*; fn run() { parent_call(); }",
            ),
            ("src/target.rs", "pub fn call() {}"),
        ],
        "src/lib.rs",
    )?;

    for symbol in [
        "renamed",
        "grouped_target::call",
        "grouped_call",
        "self_call",
    ] {
        assert!(state.edges.iter().any(|edge| {
            edge.kind == "call"
                && edge.to_symbol.as_deref() == Some(symbol)
                && edge.to_path.as_deref() == Some("src/target.rs")
                && edge.lexical_resolution == Some(LexicalResolutionStatus::Local)
        }));
    }
    assert!(state.edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_symbol.as_deref() == Some("parent_call")
            && edge.to_path.as_deref() == Some("src/target.rs")
            && edge.lexical_resolution == Some(LexicalResolutionStatus::Local)
    }));
    assert!(
        discovered
            .iter()
            .any(|path| path.ends_with("src/target.rs"))
    );
    assert!(state.unresolved_edges.iter().any(|edge| {
        edge.kind == "use"
            && edge.to_path.is_none()
            && !edge.span.trim().is_empty()
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|reason| reason.contains("glob import is intentionally unresolved"))
    }));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn lexical_alias_collision_is_an_explicit_unresolved_call_edge() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "lexical-alias-collision",
        &[
            (
                "src/lib.rs",
                "mod first; mod second; use crate::first::call as same; use crate::second::call as same; fn run() { same(); }",
            ),
            ("src/first.rs", "pub fn call() {}"),
            ("src/second.rs", "pub fn call() {}"),
        ],
        "src/lib.rs",
    )?;

    assert!(state.unresolved_edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_path.is_none()
            && !edge.span.trim().is_empty()
            && edge
                .to_symbol
                .as_deref()
                .is_some_and(|reason| reason.contains("ambiguous lexical import alias `same`"))
            && edge.lexical_resolution == Some(LexicalResolutionStatus::AmbiguousAlias)
    }));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn lexical_unresolved_and_ambiguous_calls_retain_distinct_statuses() -> TestResult {
    let (unresolved, _, root) = scan_source_fixture(
        "lexical-unresolved",
        &[("src/lib.rs", "fn run() { missing(); }")],
        "src/lib.rs",
    )?;
    assert!(unresolved.unresolved_edges.iter().any(|edge| {
        edge.kind == "call" && edge.lexical_resolution == Some(LexicalResolutionStatus::Unresolved)
    }));
    cleanup_dir(root)?;

    let (ambiguous, _, root) = scan_source_fixture(
        "lexical-ambiguous",
        &[
            ("src/about_info/mod.rs", "fn run() { self::child::go(); }"),
            ("src/about_info/child.rs", "pub fn go() {}"),
            ("src/about_info/child/mod.rs", "pub fn go() {}"),
        ],
        "src/about_info/mod.rs",
    )?;
    assert!(ambiguous.unresolved_edges.iter().any(|edge| {
        edge.kind == "call" && edge.lexical_resolution == Some(LexicalResolutionStatus::Ambiguous)
    }));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn back_edge_to_scanned_target_is_not_left_unresolved() -> TestResult {
    let (state, _, root) = scan_source_fixture(
        "back-edge",
        &[
            ("src/lib.rs", "mod b; mod a;"),
            ("src/b.rs", "pub fn go() {}"),
            ("src/a.rs", "pub fn run() { crate::b::go(); }"),
        ],
        "src/lib.rs",
    )?;
    assert!(state.edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_path.as_deref() == Some("src/b.rs")
            && edge.lexical_resolution == Some(LexicalResolutionStatus::Local)
    }));
    assert!(
        state
            .unresolved_edges
            .iter()
            .all(|edge| edge.to_path.as_deref() != Some("src/b.rs"))
    );
    cleanup_dir(root)?;
    Ok(())
}
