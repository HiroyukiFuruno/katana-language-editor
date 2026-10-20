use serde::{Deserialize, Serialize};

pub(super) const SCHEMA: &str = "kle.source-anchor-catalog.v1";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceAnchorCatalog {
    pub(super) schema: String,
    pub(super) source_closure_sha256: String,
    pub(super) root: super::super::root_model::ManifestRoot,
    pub(super) alias_ledger_sha256: String,
    pub(super) files: Vec<AnchoredFile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AnchoredFile {
    pub(super) path: String,
    pub(super) sha256: String,
    pub(super) source_utf8: String,
    pub(super) anchors: Vec<AnchorFact>,
    pub(super) unresolved_syntax: Vec<UnresolvedSyntax>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AnchorFact {
    pub(super) relation: AnchorRelation,
    pub(super) requirement_id: String,
    pub(super) short_reference: String,
    pub(super) kind: AnchorKind,
    pub(super) owner: String,
    pub(super) name: String,
    pub(super) span: SourceSpan,
    pub(super) node_sha256: String,
    pub(super) excerpt_sha256: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnchorRelation {
    ReferenceCandidate,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum AnchorKind {
    EnumVariant,
    FunctionDeclaration,
    TestDeclaration,
    StructConstruction,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct SourceSpan {
    pub(super) start_byte: usize,
    pub(super) end_byte: usize,
    pub(super) start_line: usize,
    /// Unicode文字数や表示幅ではなく、行内のUTF-8 byte offsetを保持する。
    pub(super) start_column: usize,
    pub(super) end_line: usize,
    /// Unicode文字数や表示幅ではなく、行内のUTF-8 byte offsetを保持する。
    pub(super) end_column: usize,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct UnresolvedSyntax {
    pub(super) owner: String,
    pub(super) span: SourceSpan,
    pub(super) syntax_sha256: String,
    pub(super) reason: String,
}

pub(super) type ScanFacts = (Vec<AnchorFact>, Vec<UnresolvedSyntax>);
