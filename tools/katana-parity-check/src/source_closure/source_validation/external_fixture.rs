use crate::source_closure::edge_model::{
    ExternalUiDependency, ExternalUiInvocationEdge, ExternalUiSymbolSpan, SourceFileHash,
};
use crate::source_closure::fingerprint::sha256_hex;

const SHA256_HEX_LENGTH: usize = 64;

fn source_files() -> Vec<SourceFileHash> {
    vec![SourceFileHash {
        path: "src/lib.rs".into(),
        sha256: "c".repeat(SHA256_HEX_LENGTH),
    }]
}

fn symbol_spans() -> Vec<ExternalUiSymbolSpan> {
    vec![ExternalUiSymbolSpan {
        symbol: "TextEdit::multiline".into(),
        span: "egui:src/lib.rs:1:1".into(),
    }]
}

fn invocation_edges() -> Vec<ExternalUiInvocationEdge> {
    vec![ExternalUiInvocationEdge {
        kind: "call".into(),
        source_file: "src/editor.rs".into(),
        span: "katana:src/editor.rs:1:1".into(),
        katana_symbol: "editor::run".into(),
        dependency_symbol: "TextEdit::multiline".into(),
    }]
}

pub(super) fn dependency(package: &str) -> Result<ExternalUiDependency, serde_json::Error> {
    rebuild_fingerprint(ExternalUiDependency {
        package: package.into(),
        version: "0.34.0".into(),
        source: "registry+https://example.invalid/index".into(),
        package_checksum: "a".repeat(SHA256_HEX_LENGTH),
        lock_sha256: "b".repeat(SHA256_HEX_LENGTH),
        source_files: source_files(),
        symbols: vec!["TextEdit::multiline".into()],
        symbol_spans: symbol_spans(),
        katana_invoking_edges: invocation_edges(),
        direct_semantic_edges: Vec::new(),
        replacement_leafs: vec!["editor.text_edit".into()],
        unresolved_evidence: Vec::new(),
        complete: true,
        fingerprint: String::new(),
    })
}

pub(super) fn rebuild_fingerprint(
    mut dependency: ExternalUiDependency,
) -> Result<ExternalUiDependency, serde_json::Error> {
    dependency.fingerprint.clear();
    dependency.fingerprint = sha256_hex(&serde_json::to_vec(&dependency)?);
    Ok(dependency)
}
