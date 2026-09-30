use serde::{Deserialize, Serialize};

use super::super::model::LexicalResolutionStatus;
use super::super::model::ManifestRoot;

pub(super) const BRANCH_SPAN_PRECISION: &str = "line_level_only; same_line_nesting_not_proven";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceActionFact {
    pub(crate) path: String,
    pub(crate) sha256: String,
    pub(crate) span: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceActionRouteCandidate {
    pub(crate) action_variant: String,
    pub(crate) definition: SourceActionFact,
    pub(crate) construction: SourceActionFact,
    pub(crate) dispatch: SourceActionFact,
    pub(crate) dispatch_variant_pattern: SourceActionFact,
    pub(crate) requirement_binding_ids: Vec<String>,
    pub(crate) branch_span_precision: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceActionUnresolved {
    pub(crate) kind: String,
    pub(crate) source_edge_kind: Option<String>,
    pub(crate) source_edge_detail: Option<String>,
    pub(crate) source_edge_resolution: Option<LexicalResolutionStatus>,
    pub(crate) action_variant: Option<String>,
    pub(crate) path: String,
    pub(crate) span: Option<String>,
    pub(crate) reason: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceActionBindingReport {
    pub(crate) root: ManifestRoot,
    pub(crate) requirement_binding_fingerprint: String,
    pub(crate) branch_span_precision: String,
    pub(crate) candidates: Vec<SourceActionRouteCandidate>,
    pub(crate) unresolved: Vec<SourceActionUnresolved>,
    pub(crate) unmatched_requirement_binding_ids: Vec<String>,
    pub(crate) fingerprint: String,
}

impl SourceActionBindingReport {
    pub(crate) fn validate_branch_span_precision(&self) -> Result<(), String> {
        if self.branch_span_precision != BRANCH_SPAN_PRECISION
            || self
                .candidates
                .iter()
                .any(|route| route.branch_span_precision != BRANCH_SPAN_PRECISION)
        {
            return Err("source action binding span precision is invalid".into());
        }
        Ok(())
    }

    pub(crate) fn validate_fingerprint(&self) -> Result<(), String> {
        if self.fingerprint_value()? != self.fingerprint {
            return Err("source action binding fingerprint mismatch".into());
        }
        Ok(())
    }

    pub(crate) fn fingerprint_value(&self) -> Result<String, String> {
        let digest = FingerprintInput {
            root: &self.root,
            requirement_binding_fingerprint: &self.requirement_binding_fingerprint,
            branch_span_precision: &self.branch_span_precision,
            candidates: &self.candidates,
            unresolved: &self.unresolved,
            unmatched_requirement_binding_ids: &self.unmatched_requirement_binding_ids,
        };
        let bytes = serde_json::to_vec(&digest)
            .map_err(|error| format!("serialize source action binding fingerprint: {error}"))?;
        Ok(super::super::fingerprint::sha256_hex(&bytes))
    }
}

#[derive(Serialize)]
pub(super) struct FingerprintInput<'a> {
    pub(super) root: &'a ManifestRoot,
    pub(super) requirement_binding_fingerprint: &'a str,
    pub(super) branch_span_precision: &'a str,
    pub(super) candidates: &'a [SourceActionRouteCandidate],
    pub(super) unresolved: &'a [SourceActionUnresolved],
    pub(super) unmatched_requirement_binding_ids: &'a [String],
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ParsedSpan {
    pub(super) file: String,
    pub(super) start_line: usize,
    pub(super) start_column: usize,
    pub(super) end_line: usize,
    pub(super) end_column: usize,
}

pub(super) struct IndexedDefinition {
    pub(super) fact: SourceActionFact,
}

pub(super) struct IndexedConstruction {
    pub(super) variant: String,
    pub(super) path: String,
    pub(super) fact: SourceActionFact,
    pub(super) span: ParsedSpan,
}

pub(super) struct IndexedDispatch {
    pub(super) action_variant: String,
    pub(super) path: String,
    pub(super) span: String,
    pub(super) fact: SourceActionFact,
    pub(super) variant_pattern: SourceActionFact,
    pub(super) unresolved_reasons: Vec<String>,
}
