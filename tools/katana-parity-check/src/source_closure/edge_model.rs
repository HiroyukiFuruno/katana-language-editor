use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LexicalResolutionStatus {
    Local,
    Unresolved,
    Ambiguous,
    AmbiguousAlias,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ClosureEdge {
    pub(crate) id: String,
    pub(crate) from_file: String,
    pub(crate) kind: String,
    pub(crate) from_symbol: String,
    pub(crate) to_path: Option<String>,
    pub(crate) to_symbol: Option<String>,
    pub(crate) lexical_resolution: Option<LexicalResolutionStatus>,
    pub(crate) span: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SourceClosureFile {
    pub(crate) path: String,
    pub(crate) sha256: String,
    pub(crate) incoming_edges: Vec<ClosureEdge>,
    pub(crate) classification: String,
    pub(crate) classification_rationale: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExternalUiDependency {
    pub(crate) package: String,
    pub(crate) version: String,
    pub(crate) source: String,
    pub(crate) package_checksum: String,
    pub(crate) lock_sha256: String,
    pub(crate) source_files: Vec<SourceFileHash>,
    pub(crate) symbols: Vec<String>,
    pub(crate) symbol_spans: Vec<ExternalUiSymbolSpan>,
    pub(crate) katana_invoking_edges: Vec<ExternalUiInvocationEdge>,
    pub(crate) direct_semantic_edges: Vec<ExternalUiDirectSemanticEdge>,
    pub(crate) replacement_leafs: Vec<String>,
    pub(crate) unresolved_evidence: Vec<String>,
    pub(crate) complete: bool,
    pub(crate) fingerprint: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExternalUiInvocationEdge {
    pub(crate) kind: String,
    pub(crate) source_file: String,
    pub(crate) span: String,
    pub(crate) katana_symbol: String,
    pub(crate) dependency_symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExternalUiDirectSemanticEdge {
    pub(crate) from_symbol: String,
    pub(crate) kind: String,
    pub(crate) source_file: String,
    pub(crate) span: String,
    pub(crate) target_symbol: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct SourceFileHash {
    pub(crate) path: String,
    pub(crate) sha256: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(crate) struct ExternalUiSymbolSpan {
    pub(crate) symbol: String,
    pub(crate) span: String,
}
