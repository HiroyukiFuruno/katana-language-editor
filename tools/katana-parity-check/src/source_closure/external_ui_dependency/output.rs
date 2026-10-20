use serde_json;

use super::super::edge_model::{
    ExternalUiDependency, ExternalUiDirectSemanticEdge, ExternalUiInvocationEdge,
    ExternalUiSymbolSpan, SourceFileHash,
};
use super::super::fingerprint::sha256_hex;
use super::lock::LockPackage;
use super::{PACKAGE, REQUIRED};

const TRANSITIVE_SEMANTIC_CLOSURE_REASON: &str =
    "transitive semantic closure has not been captured";

pub(super) fn finish(
    lock: LockPackage,
    source_files: Vec<SourceFileHash>,
    symbols: Vec<String>,
    symbol_spans: Vec<ExternalUiSymbolSpan>,
    direct_semantic_edges: Vec<ExternalUiDirectSemanticEdge>,
    edges: Vec<ExternalUiInvocationEdge>,
    mut unresolved: Vec<String>,
) -> ExternalUiDependency {
    unresolved.push(TRANSITIVE_SEMANTIC_CLOSURE_REASON.into());
    unresolved.sort();
    unresolved.dedup();
    let complete = unresolved.is_empty()
        && REQUIRED
            .iter()
            .all(|required| edges.iter().any(|edge| edge.dependency_symbol == *required))
        && !source_files.is_empty()
        && !lock.version.is_empty()
        && !lock.source.is_empty()
        && !lock.checksum.is_empty();
    let mut result = ExternalUiDependency {
        package: PACKAGE.into(),
        version: lock.version,
        source: lock.source,
        package_checksum: lock.checksum,
        lock_sha256: lock.lock_sha256,
        source_files,
        symbols,
        symbol_spans,
        katana_invoking_edges: edges,
        direct_semantic_edges,
        replacement_leafs: Vec::new(),
        unresolved_evidence: unresolved,
        complete,
        fingerprint: String::new(),
    };
    let bytes = serde_json::to_vec(&result).unwrap_or_default();
    result.fingerprint = sha256_hex(&bytes);
    result
}

#[cfg(test)]
#[path = "output_tests.rs"]
mod output_tests;
