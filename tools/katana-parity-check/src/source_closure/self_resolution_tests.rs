use super::*;
use crate::source_closure::model::LexicalResolutionStatus;

#[test]
fn inherent_self_static_method_is_an_exact_local_call_edge() -> TestResult {
    let (state, discovered, root) = scan_source_fixture(
        "inherent-self-static-method",
        &[(
            "src/lib.rs",
            "struct Editor; impl Editor { fn run() { Self::apply(); } fn apply() {} }",
        )],
        "src/lib.rs",
    )?;

    assert!(state.edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_symbol.as_deref() == Some("Self::apply")
            && edge.to_path.as_deref() == Some("src/lib.rs")
            && edge.lexical_resolution == Some(LexicalResolutionStatus::Local)
    }));
    assert!(discovered.iter().any(|path| path.ends_with("src/lib.rs")));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn trait_impl_self_static_method_remains_unresolved() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "trait-impl-self-static-method",
        &[(
            "src/lib.rs",
            "trait Runner { fn run(); } struct Editor; impl Editor { fn apply() {} } impl Runner for Editor { fn run() { Self::apply(); } }",
        )],
        "src/lib.rs",
    )?;

    assert!(state.unresolved_edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_symbol.as_deref().is_some_and(|symbol| {
                symbol.contains("Self::apply")
                    && symbol.contains("not an exact inherent member in the current impl")
            })
            && edge.lexical_resolution == Some(LexicalResolutionStatus::Unresolved)
    }));
    cleanup_dir(root)?;
    Ok(())
}

#[test]
fn inherent_self_associated_const_is_an_exact_local_edge() -> TestResult {
    let (state, _discovered, root) = scan_source_fixture(
        "inherent-self-associated-const",
        &[(
            "src/lib.rs",
            "struct Editor; impl Editor { const ID: u8 = 1; fn run() { let _ = Self::ID; } }",
        )],
        "src/lib.rs",
    )?;

    assert!(state.edges.iter().any(|edge| {
        edge.kind == "call"
            && edge.to_symbol.as_deref() == Some("Self::ID")
            && edge.to_path.as_deref() == Some("src/lib.rs")
            && edge.lexical_resolution == Some(LexicalResolutionStatus::Local)
    }));
    cleanup_dir(root)?;
    Ok(())
}
