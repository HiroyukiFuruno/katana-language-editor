use std::collections::BTreeSet;
use std::path::Path;

use syn::visit::Visit;

use super::*;

fn lock(bytes: &[u8]) -> Result<LockPackage, String> {
    let mut unresolved = Vec::new();
    parse_lock(bytes, &mut unresolved).ok_or_else(|| {
        format!("lock fixture did not resolve a unique egui package: {unresolved:?}")
    })
}

fn source(text: &str) -> Vec<(String, String)> {
    vec![("src/editor.rs".into(), text.into())]
}

#[test]
fn identical_inputs_produce_identical_bytes() -> Result<(), Box<dyn std::error::Error>> {
    let lock_bytes = br#"[[package]]
name = "egui"
version = "0.34.0"
source = "registry+https://example.invalid/index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;
    let (edges, unresolved) = scan_katana_sources(&source(
        "fn edit(ctx: &egui::Context) { let text_edit = TextEdit::multiline(\"x\"); text_edit.show(ui); let _ = TextEdit::load_state(); TextEdit::store_state(); let _ = Event::Paste; ctx.input_mut(|i| i.consume_shortcut(&parsed)); }",
    ));
    let first = finish(
        lock(lock_bytes)?,
        vec![],
        vec![],
        vec![],
        vec![],
        edges.clone(),
        unresolved.clone(),
    );
    let second = finish(
        lock(lock_bytes)?,
        vec![],
        vec![],
        vec![],
        vec![],
        edges,
        unresolved,
    );
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    Ok(())
}

#[test]
fn lock_source_and_span_mutations_change_fingerprint() -> Result<(), Box<dyn std::error::Error>> {
    let base = br#"[[package]]
name = "egui"
version = "0.34.0"
source = "registry+https://example.invalid/index"
checksum = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
"#;
    let changed_version = String::from_utf8(base.to_vec())?.replace("0.34.0", "0.34.1");
    let changed_source =
        String::from_utf8(base.to_vec())?.replace("example.invalid", "example.test");
    let (edges, unresolved) =
        scan_katana_sources(&source("fn edit() { TextEdit::multiline(\"x\"); }"));
    let first = finish(
        lock(base)?,
        vec![],
        vec![],
        vec![],
        vec![],
        edges.clone(),
        unresolved.clone(),
    );
    let version = finish(
        lock(changed_version.as_bytes())?,
        vec![],
        vec![],
        vec![],
        vec![],
        edges.clone(),
        unresolved.clone(),
    );
    let registry = finish(
        lock(changed_source.as_bytes())?,
        vec![],
        vec![],
        vec![],
        vec![],
        edges,
        unresolved,
    );
    assert_ne!(first.fingerprint, version.fingerprint);
    assert_ne!(first.fingerprint, registry.fingerprint);
    let (changed_edges, changed_unresolved) =
        scan_katana_sources(&source("fn edit() {\n TextEdit::multiline(\"x\");\n}"));
    let span = finish(
        lock(base)?,
        vec![],
        vec![],
        vec![],
        vec![],
        changed_edges,
        changed_unresolved,
    );
    assert_ne!(first.fingerprint, span.fingerprint);
    Ok(())
}

#[test]
fn missing_and_ambiguous_lock_packages_are_unresolved() {
    let mut unresolved = Vec::new();
    assert!(parse_lock(b"[[package]]\nname = \"other\"\n", &mut unresolved).is_none());
    assert!(unresolved.iter().any(|reason| reason.contains("missing")));
    let mut unresolved = Vec::new();
    assert!(parse_lock(b"[[package]]\nname = \"egui\"\nversion = \"1\"\n\n[[package]]\nname = \"egui\"\nversion = \"2\"\n", &mut unresolved).is_none());
    assert!(unresolved.iter().any(|reason| reason.contains("2")));
}

#[test]
fn exact_invocation_edges_are_recorded_with_symbols_and_spans() {
    let (edges, unresolved) = scan_katana_sources(&source(
        "mod editor { use egui::Context; fn run(context: &Context) { let text_edit = TextEdit::multiline(\"x\"); text_edit.show(ui); TextEdit::load_state(); TextEdit::store_state(); let _paste = Event::Paste; context.input_mut(|input| input.consume_shortcut(&parsed)); } }",
    ));
    assert!(unresolved.is_empty(), "{unresolved:?}");
    assert_eq!(edges.len(), REQUIRED.len());
    assert!(edges.iter().all(|edge| edge.source_file == "src/editor.rs"
        && edge.katana_symbol == "editor::run"
        && edge.span.starts_with("katana:src/editor.rs:")));
    assert_eq!(
        edges
            .iter()
            .map(|edge| edge.dependency_symbol.as_str())
            .collect::<BTreeSet<_>>()
            .len(),
        REQUIRED.len()
    );
}

#[test]
fn text_edit_show_requires_an_unshadowed_nonclosure_builder_binding() {
    for snippet in [
        "fn run() { let text_edit = TextEdit::multiline(text); let text_edit = Other::new(); text_edit.show(ui); }",
        "fn run() { let text_edit = TextEdit::multiline(text); let _deferred = || text_edit.show(ui); }",
        "fn run() { let other = Other::new(); other.show(ui); }",
    ] {
        let (edges, _) = scan_katana_sources(&source(snippet));
        assert!(
            !edges
                .iter()
                .any(|edge| edge.dependency_symbol == "TextEdit::show"),
            "unexpected TextEdit::show edge for {snippet}"
        );
    }
}

#[test]
fn text_edit_show_accepts_a_builder_from_the_same_closure() {
    let (edges, _) = scan_katana_sources(&source(
        "fn run() { panel.show(ui, |ui| { let text_edit = TextEdit::multiline(text); text_edit.show(ui); }); }",
    ));
    assert!(
        edges
            .iter()
            .any(|edge| edge.dependency_symbol == "TextEdit::show")
    );
}

#[test]
fn direct_context_consume_shortcut_does_not_satisfy_required_edge() {
    let (edges, unresolved) = scan_katana_sources(&source(
        "fn run(context: &egui::Context) { context.consume_shortcut(&parsed); }",
    ));
    assert!(
        !edges
            .iter()
            .any(|edge| edge.dependency_symbol == "InputState::consume_shortcut")
    );
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("immediate input_mut closure binding"))
    );
}

#[test]
fn missing_or_unreadable_katana_evidence_is_unresolved() {
    let (sources, unresolved) = read_katana_sources(
        Path::new("/definitely/missing/katana-root"),
        &["src/missing.rs".into()],
    );
    assert!(sources.is_empty());
    assert!(
        unresolved
            .iter()
            .any(|reason| reason.contains("KatanA invocation source unreadable"))
    );
}

#[test]
fn dependency_comments_and_strings_do_not_define_required_symbols()
-> Result<(), Box<dyn std::error::Error>> {
    let file = syn::parse_file(
        r#"
            /* WHY: comments and strings are not AST definitions. */
            const TEXT: &str = "TextEdit::load_state TextEdit::store_state";
        "#,
    )?;
    let mut visitor = DependencyDefinitionVisitor::default();
    visitor.visit_file(&file);
    assert!(visitor.symbols.is_empty());
    Ok(())
}

#[test]
fn diagnostic_records_supplied_invocations_without_claiming_semantic_completion() {
    let sources = source(
        "fn edit(ctx: &egui::Context) { let text_edit = TextEdit::multiline(text); text_edit.show(ui); TextEdit::load_state(ctx, id); TextEdit::store_state(ctx, id, state); let _ = Event::Paste; ctx.input_mut(|i| i.consume_shortcut(&shortcut)); }",
    );
    let report = audit_locked_sources(Path::new("/missing-reference"), b"invalid = [", &sources);
    assert_eq!(report.katana_invoking_edges.len(), REQUIRED.len());
    assert!(!report.complete);
    assert!(report.source_files.is_empty());
    assert!(report.replacement_leafs.is_empty());
    assert!(
        report
            .unresolved_evidence
            .iter()
            .any(|reason| reason.contains("parse failure"))
    );
    assert!(
        report
            .unresolved_evidence
            .iter()
            .any(|reason| reason.contains("does not capture transitive semantic closure"))
    );
    let repeated = audit_locked_sources(Path::new("/missing-reference"), b"invalid = [", &sources);
    assert_eq!(report.fingerprint, repeated.fingerprint);
    let empty = audit_locked_sources(Path::new("/missing-reference"), b"invalid = [", &[]);
    assert_ne!(report.fingerprint, empty.fingerprint);
    assert!(empty.katana_invoking_edges.is_empty());
    assert!(
        empty
            .unresolved_evidence
            .iter()
            .any(|reason| reason.contains("missing exact KatanA invocation"))
    );
}
