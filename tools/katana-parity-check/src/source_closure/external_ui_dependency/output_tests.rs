use super::super::super::edge_model::{
    ExternalUiDirectSemanticEdge, ExternalUiInvocationEdge, ExternalUiSymbolSpan, SourceFileHash,
};
use super::super::super::fingerprint::sha256_hex;
use super::super::REQUIRED;
use super::super::lock::LockPackage;
use super::finish;

const SHA256_HEX_LENGTH: usize = 64;

fn required_edges() -> Vec<ExternalUiInvocationEdge> {
    REQUIRED
        .iter()
        .map(|dependency_symbol| ExternalUiInvocationEdge {
            kind: "call".into(),
            source_file: "src/editor.rs".into(),
            span: format!("katana:src/editor.rs:{dependency_symbol}"),
            katana_symbol: "editor::run".into(),
            dependency_symbol: (*dependency_symbol).into(),
        })
        .collect()
}

fn required_symbols() -> Vec<String> {
    REQUIRED.iter().map(|symbol| (*symbol).into()).collect()
}

fn required_spans() -> Vec<ExternalUiSymbolSpan> {
    REQUIRED
        .iter()
        .map(|symbol| ExternalUiSymbolSpan {
            symbol: (*symbol).into(),
            span: format!("egui:src/lib.rs:{symbol}"),
        })
        .collect()
}

fn direct_semantic_edges() -> Vec<ExternalUiDirectSemanticEdge> {
    vec![ExternalUiDirectSemanticEdge {
        from_symbol: "TextEdit::show".into(),
        kind: "call".into(),
        source_file: "egui/src/widgets/text_edit.rs".into(),
        span: "egui:src/widgets/text_edit.rs:1:1-1:22".into(),
        target_symbol: "TextEditState::load".into(),
    }]
}

struct CompleteInputs {
    lock: LockPackage,
    files: Vec<SourceFileHash>,
    symbols: Vec<String>,
    spans: Vec<ExternalUiSymbolSpan>,
    edges: Vec<ExternalUiInvocationEdge>,
}

fn complete_inputs() -> CompleteInputs {
    CompleteInputs {
        lock: LockPackage {
            version: "0.34.0".into(),
            source: "registry+https://example.invalid/index".into(),
            checksum: "a".repeat(SHA256_HEX_LENGTH),
            lock_sha256: "b".repeat(SHA256_HEX_LENGTH),
        },
        files: vec![SourceFileHash {
            path: "src/lib.rs".into(),
            sha256: "c".repeat(SHA256_HEX_LENGTH),
        }],
        symbols: required_symbols(),
        spans: required_spans(),
        edges: required_edges(),
    }
}

#[test]
fn complete_inputs_remain_incomplete_without_transitive_semantic_closure() {
    let inputs = complete_inputs();
    let result = finish(
        inputs.lock,
        inputs.files,
        inputs.symbols,
        inputs.spans,
        Vec::new(),
        inputs.edges,
        Vec::new(),
    );

    assert!(!result.complete);
    assert_eq!(
        result.unresolved_evidence,
        vec!["transitive semantic closure has not been captured"]
    );
}

#[test]
fn unresolved_evidence_is_sorted_and_deduplicated_with_the_common_reason() {
    let inputs = complete_inputs();
    let result = finish(
        inputs.lock,
        inputs.files,
        inputs.symbols,
        inputs.spans,
        Vec::new(),
        inputs.edges,
        vec![
            "z unresolved".into(),
            "transitive semantic closure has not been captured".into(),
            "a unresolved".into(),
            "z unresolved".into(),
        ],
    );

    assert_eq!(
        result.unresolved_evidence,
        vec![
            "a unresolved",
            "transitive semantic closure has not been captured",
            "z unresolved",
        ]
    );
}

#[test]
fn fingerprint_matches_the_final_content() -> Result<(), Box<dyn std::error::Error>> {
    let inputs = complete_inputs();
    let result = finish(
        inputs.lock,
        inputs.files,
        inputs.symbols,
        inputs.spans,
        Vec::new(),
        inputs.edges,
        vec!["input unresolved".into()],
    );
    let mut content = result.clone();
    let fingerprint = content.fingerprint.clone();
    content.fingerprint.clear();

    assert_eq!(fingerprint, sha256_hex(&serde_json::to_vec(&content)?));
    Ok(())
}

#[test]
fn direct_semantic_edges_are_fingerprint_inputs() {
    let inputs = complete_inputs();
    let without_direct_edge = finish(
        inputs.lock.clone(),
        inputs.files.clone(),
        inputs.symbols.clone(),
        inputs.spans.clone(),
        Vec::new(),
        inputs.edges.clone(),
        Vec::new(),
    );
    let with_direct_edge = finish(
        inputs.lock,
        inputs.files,
        inputs.symbols,
        inputs.spans,
        direct_semantic_edges(),
        inputs.edges,
        Vec::new(),
    );

    assert_ne!(
        without_direct_edge.fingerprint,
        with_direct_edge.fingerprint
    );
}
