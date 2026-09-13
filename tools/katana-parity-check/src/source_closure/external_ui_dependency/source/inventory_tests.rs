use std::collections::BTreeMap;

use super::super::super::edge_model::SourceFileHash;
use super::super::super::fingerprint::sha256_hex;
use super::parse_sources;

const ENTRIES: &str = "struct TextEdit; impl TextEdit { fn multiline() {} fn load_state() {} fn store_state() {} } enum Event { Paste } struct InputState; impl InputState { fn consume_shortcut() {} }";

fn parse(
    sources: BTreeMap<String, Vec<u8>>,
) -> (Vec<SourceFileHash>, Vec<String>, Vec<String>, usize) {
    let mut unresolved = Vec::new();
    let (files, symbols, spans, direct_semantic_edges) = parse_sources(sources, &mut unresolved);
    assert!(unresolved.is_empty(), "{unresolved:?}");
    (
        files,
        symbols.into_iter().collect(),
        spans.into_iter().map(|span| span.span).collect(),
        direct_semantic_edges.len(),
    )
}

fn sources(entries: &[(&str, &[u8])]) -> BTreeMap<String, Vec<u8>> {
    entries
        .iter()
        .map(|(path, bytes)| ((*path).into(), bytes.to_vec()))
        .collect()
}

#[test]
fn parse_sources_keeps_entryless_unicode_crlf_file_hash_without_symbols() {
    let base = sources(&[("z_entries.rs", ENTRIES.as_bytes())]);
    let (expected_files, expected_symbols, expected_spans, expected_direct_edges) =
        parse(base.clone());
    let extra = "pub struct 日本語;\r\n".as_bytes().to_vec();
    let mut with_extra = base;
    with_extra.insert("a_unicode.rs".into(), extra.clone());
    let (files, symbols, spans, direct_edges) = parse(with_extra);
    assert_eq!(symbols, expected_symbols);
    assert_eq!(spans, expected_spans);
    assert_eq!(direct_edges, expected_direct_edges);
    assert_eq!(files[0].path, "egui/a_unicode.rs");
    assert_eq!(files[0].sha256, sha256_hex(&extra));
    assert_eq!(files.len(), expected_files.len() + 1);
}

#[test]
fn parse_sources_sorts_paths_and_is_deterministic() -> Result<(), serde_json::Error> {
    let input = sources(&[
        ("z.rs", ENTRIES.as_bytes()),
        ("m.rs", "pub struct 中間;\n".as_bytes()),
        ("a.rs", b"pub struct Alpha;\n"),
    ]);
    let first = parse(input.clone());
    let second = parse(input);
    assert_eq!(serde_json::to_vec(&first)?, serde_json::to_vec(&second)?);
    assert_eq!(
        first
            .0
            .iter()
            .map(|file| file.path.as_str())
            .collect::<Vec<_>>(),
        vec!["egui/a.rs", "egui/m.rs", "egui/z.rs"]
    );
    Ok(())
}

#[test]
fn parse_sources_keeps_broken_ast_unresolved_without_success_evidence() {
    let mut unresolved = Vec::new();
    let (files, symbols, spans, direct_semantic_edges) = parse_sources(
        sources(&[("broken.rs", b"struct Broken {")]),
        &mut unresolved,
    );
    assert!(
        files.is_empty()
            && symbols.is_empty()
            && spans.is_empty()
            && direct_semantic_edges.is_empty()
    );
    assert!(
        unresolved
            .iter()
            .any(|item| item.contains("AST parse failure: broken.rs")),
        "{unresolved:?}"
    );
}
